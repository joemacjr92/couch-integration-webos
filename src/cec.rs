//! Hierarchical HDMI-CEC input discovery for LG webOS.
//!
//! The TV's external-input list remains the authoritative top-level selector.
//! EIM sub-lists add distinct downstream CEC devices. A depth-1 device whose
//! TV-reported name matches its physical input label is already represented by
//! that top-level input and is not added a second time.

use couch_sdk::Selectable;
use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub(crate) const INPUT_PREFIX: &str = "cec/";

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CecSelection {
    pub(crate) parent_id: String,
    pub(crate) parent_app_id: String,
    pub(crate) unique_id: String,
}

fn scalar(value: &Value) -> Option<String> {
    match value {
        Value::String(text) => {
            let text = text.trim();
            (!text.is_empty() && !text.chars().any(char::is_control))
                .then(|| text.to_owned())
        }
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

fn identity(row: &Value) -> Option<(String, String)> {
    let vendor = scalar(row.get("vendorId")?)?;
    let address = scalar(row.get("physicalAddress")?)?;
    if address == "0" || address == "0x0" || address == "0x0000" {
        return None;
    }
    Some((vendor, address))
}

fn child_id(parent_id: &str, unique_id: &str, identity: Option<&(String, String)>) -> String {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(parent_id.as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(unique_id.as_bytes());
    if let Some((vendor, address)) = identity {
        bytes.push(0);
        bytes.extend_from_slice(vendor.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(address.as_bytes());
    }
    let encoded: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    format!("{INPUT_PREFIX}{encoded}")
}

fn parent_context(parent_id: &str, parent_label: &str) -> String {
    if let Some(port) = parent_id.strip_prefix("HDMI_") {
        if !port.is_empty() && port.chars().all(|ch| ch.is_ascii_alphanumeric()) {
            return format!("HDMI {port}");
        }
    }
    if !parent_label.trim().is_empty() {
        parent_label.trim().to_owned()
    } else {
        parent_id.to_owned()
    }
}

fn same_tv_name(left: &str, right: &str) -> bool {
    left.trim().eq_ignore_ascii_case(right.trim())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty() && name.len() <= 4096 && !name.chars().any(char::is_control)
}

/// Append EIM descendants to the existing flat Couch input list.
///
/// The physical input rows are never replaced or renamed. Only children with
/// a valid parent, launch-point app ID and EIM uniqueId are added. Duplicate
/// CEC reports are identified by the TV's vendor/physical-address pair, not
/// by display name or uniqueId (which can differ between duplicate reports).
pub(crate) fn append_children(
    external_rows: &[Value],
    eim_rows: Option<&[Value]>,
    inputs: &mut Vec<Selectable>,
    selections: &mut HashMap<String, CecSelection>,
) {
    let Some(eim_rows) = eim_rows else {
        return;
    };

    for eim_parent in eim_rows {
        let Some(parent_id) = eim_parent.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some(external_parent) = external_rows
            .iter()
            .find(|row| row.get("id").and_then(Value::as_str) == Some(parent_id))
        else {
            continue;
        };
        let Some(parent_app_id) = external_parent
            .get("appId")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
        else {
            continue;
        };
        let parent_label = external_parent
            .get("label")
            .and_then(Value::as_str)
            .unwrap_or(parent_id)
            .trim();
        let context = parent_context(parent_id, parent_label);
        let Some(children) = eim_parent.get("subList").and_then(Value::as_array) else {
            continue;
        };

        let mut seen_identities = HashSet::new();
        for child in children {
            let Some(unique_id) = child.get("uniqueId").and_then(scalar) else {
                continue;
            };
            let Some(depth) = child.get("depth").and_then(Value::as_u64) else {
                continue;
            };
            let name = child
                .get("osdName")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|name| valid_name(name))
                .unwrap_or(unique_id.as_str());

            let device_identity = identity(child);
            if let Some(identity) = &device_identity {
                if !seen_identities.insert(identity.clone()) {
                    continue;
                }
            }

            // A depth-1 CEC entry carrying the same name as the TV's physical
            // input label is the device already represented by that input.
            // This is topology/name driven; no vendor or model is special-cased.
            if depth == 1 && same_tv_name(name, parent_label) {
                continue;
            }

            let id = child_id(parent_id, &unique_id, device_identity.as_ref());
            if id.len() > 128
                || !couch_sdk::couch_model::commands::valid_input_id(&id)
                || selections.contains_key(&id)
                || inputs.len() >= 128
            {
                continue;
            }

            let display_name = format!("{name} · {context}");
            if !valid_name(&display_name) {
                continue;
            }

            inputs.push(Selectable::new(&id, display_name));
            selections.insert(
                id,
                CecSelection {
                    parent_id: parent_id.to_owned(),
                    parent_app_id: parent_app_id.to_owned(),
                    unique_id,
                },
            );
        }
    }
}

/// The EIM status response's last selected CEC uniqueId for one physical input.
pub(crate) fn last_unique_id(response: &Value, parent_id: &str) -> Option<String> {
    response
        .get("devices")?
        .as_array()?
        .iter()
        .find(|row| row.get("id").and_then(Value::as_str) == Some(parent_id))?
        .get("lastUniqueId")
        .and_then(scalar)
}

/// Exact launch request used by the verified v0.3.4 diagnostic harness.
pub(crate) fn launch_payload(selection: &CecSelection) -> Value {
    serde_json::json!({
        "id": selection.parent_app_id,
        "params": {
            "id": "uniqueId",
            "value": selection.unique_id
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> (Vec<Value>, Vec<Value>) {
        let external = vec![json!({
            "id": "HDMI_2",
            "label": "DRAGON",
            "appId": "com.webos.app.hdmi2"
        })];
        let eim = vec![json!({
            "id": "HDMI_2",
            "lastUniqueId": 8,
            "subList": [
                {"osdName":"DRAGON","uniqueId":3,"vendorId":805705,"physicalAddress":8192,"depth":1},
                {"osdName":"p3i","uniqueId":4,"vendorId":57489,"physicalAddress":8704,"depth":2},
                {"osdName":"DRAGON","uniqueId":5,"vendorId":805705,"physicalAddress":8192,"depth":1},
                {"osdName":"Fire TV Cube","uniqueId":8,"vendorId":0,"physicalAddress":8960,"depth":2}
            ]
        })];
        (external, eim)
    }

    fn collect(external: &[Value], eim: &[Value]) -> (Vec<Selectable>, HashMap<String, CecSelection>) {
        let mut inputs = vec![Selectable::new("HDMI_2", "DRAGON")];
        let mut selections = HashMap::new();
        append_children(external, Some(eim), &mut inputs, &mut selections);
        (inputs, selections)
    }

    #[test]
    fn preserves_top_level_input_and_adds_only_distinct_downstream_devices() {
        let (external, eim) = fixture();
        let (inputs, selections) = collect(&external, &eim);

        assert_eq!(inputs[0], Selectable::new("HDMI_2", "DRAGON"));
        assert_eq!(inputs.len(), 3);
        assert!(inputs.iter().any(|item| item.name == "p3i · HDMI 2"));
        assert!(inputs.iter().any(|item| item.name == "Fire TV Cube · HDMI 2"));
        assert!(!inputs.iter().any(|item| item.name.starts_with("DRAGON ·")));
        assert_eq!(selections.len(), 2);
        assert!(selections.values().any(|item| item.unique_id == "4"));
        assert!(selections.values().any(|item| item.unique_id == "8"));
    }

    #[test]
    fn duplicate_identity_is_not_based_on_unique_id_or_display_name() {
        let external = vec![json!({"id":"HDMI_3","label":"HDMI 3","appId":"com.webos.app.hdmi3"})];
        let eim = vec![json!({
            "id":"HDMI_3",
            "subList":[
                {"osdName":"Streamer","uniqueId":10,"vendorId":42,"physicalAddress":4096,"depth":2},
                {"osdName":"Renamed streamer","uniqueId":11,"vendorId":42,"physicalAddress":4096,"depth":2},
                {"osdName":"Streamer","uniqueId":12,"vendorId":43,"physicalAddress":8192,"depth":2}
            ]
        })];
        let mut inputs = vec![Selectable::new("HDMI_3", "HDMI 3")];
        let mut selections = HashMap::new();
        append_children(&external, Some(&eim), &mut inputs, &mut selections);

        assert_eq!(inputs.len(), 3);
        assert_eq!(selections.len(), 2);
        assert!(inputs.iter().any(|item| item.name == "Streamer · HDMI 3"));
        assert!(inputs.iter().any(|item| item.name == "Streamer · HDMI 3"));
    }

    #[test]
    fn matching_top_level_name_is_suppressed_only_at_depth_one() {
        let external = vec![json!({"id":"HDMI_4","label":"Receiver","appId":"com.webos.app.hdmi4"})];
        let eim = vec![json!({
            "id":"HDMI_4",
            "subList":[
                {"osdName":"Receiver","uniqueId":1,"vendorId":10,"physicalAddress":4096,"depth":1},
                {"osdName":"Receiver","uniqueId":2,"vendorId":11,"physicalAddress":8192,"depth":2}
            ]
        })];
        let mut inputs = vec![Selectable::new("HDMI_4", "Receiver")];
        let mut selections = HashMap::new();
        append_children(&external, Some(&eim), &mut inputs, &mut selections);

        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[1].name, "Receiver · HDMI 4");
        assert_eq!(selections.values().next().unwrap().unique_id, "2");
    }

    #[test]
    fn missing_identity_does_not_cause_name_based_deduplication() {
        let external = vec![json!({"id":"HDMI_1","label":"HDMI 1","appId":"com.webos.app.hdmi1"})];
        let eim = vec![json!({
            "id":"HDMI_1",
            "subList":[
                {"osdName":"Same name","uniqueId":1,"depth":2},
                {"osdName":"Same name","uniqueId":2,"depth":2}
            ]
        })];
        let mut inputs = vec![Selectable::new("HDMI_1", "HDMI 1")];
        let mut selections = HashMap::new();
        append_children(&external, Some(&eim), &mut inputs, &mut selections);

        assert_eq!(inputs.len(), 3);
        assert_eq!(selections.len(), 2);
    }

    #[test]
    fn child_ids_are_safe_stable_and_parent_scoped() {
        let left = child_id("HDMI_2", "4", Some(&("57489".into(), "8704".into())));
        let same = child_id("HDMI_2", "4", Some(&("57489".into(), "8704".into())));
        let other_parent = child_id("HDMI_3", "4", Some(&("57489".into(), "8704".into())));
        assert_eq!(left, same);
        assert_ne!(left, other_parent);
        assert!(left.len() <= 128);
        assert!(couch_sdk::couch_model::commands::valid_input_id(&left));
    }

    #[test]
    fn launch_payload_matches_the_verified_harness_contract() {
        let selection = CecSelection {
            parent_id: "HDMI_2".into(),
            parent_app_id: "com.webos.app.hdmi2".into(),
            unique_id: "4".into(),
        };
        assert_eq!(
            launch_payload(&selection),
            json!({
                "id":"com.webos.app.hdmi2",
                "params":{"id":"uniqueId","value":"4"}
            })
        );
    }

    #[test]
    fn last_unique_id_is_read_from_the_matching_parent_only() {
        let response = json!({
            "devices":[
                {"id":"HDMI_1","lastUniqueId":2},
                {"id":"HDMI_2","lastUniqueId":8}
            ]
        });
        assert_eq!(last_unique_id(&response, "HDMI_2").as_deref(), Some("8"));
        assert_eq!(last_unique_id(&response, "HDMI_3"), None);
    }
}

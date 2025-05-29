use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, Clone)]
pub struct Machine {
    #[serde(rename = "serialNumber")]
    pub serial_number: String,
    #[serde(rename = "modelName")]
    pub model: Option<String>,
    pub name: Option<String>,
    #[allow(dead_code)] // Field from API but not currently used in display
    pub location: Option<String>,
    pub connected: bool,
}

#[derive(Debug, Deserialize)]
pub struct MachinesResponse {
    pub things: Vec<Machine>,
}

#[derive(Debug, Serialize)]
pub struct MachineCommand {
    pub mode: String,
}

#[derive(Debug, Deserialize)]
pub struct MachineStatus {
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Deserialize)]
pub struct Widget {
    pub code: String,
    pub output: Option<WidgetOutput>,
}

#[derive(Debug, Deserialize)]
pub struct WidgetOutput {
    pub status: Option<String>,
    #[allow(dead_code)]
    pub mode: Option<String>,
    // Boiler-specific fields
    #[serde(rename = "readyStartTime")]
    pub ready_start_time: Option<u64>,
}

impl MachineStatus {
    pub fn is_on(&self) -> bool {
        // Look for the CMMachineStatus widget
        for widget in &self.widgets {
            if widget.code == "CMMachineStatus" {
                if let Some(output) = &widget.output {
                    if let Some(status) = &output.status {
                        return status != "StandBy";
                    }
                }
            }
        }
        false // Default to off if we can't determine the status
    }

    pub fn get_status_string(&self) -> String {
        self.get_status_string_with_time(None)
    }

    pub fn get_status_string_with_time(&self, current_time_ms: Option<u64>) -> String {
        // First, check if machine is powered on
        let mut is_powered_on = false;
        for widget in &self.widgets {
            if widget.code == "CMMachineStatus" {
                if let Some(output) = &widget.output {
                    if let Some(status) = &output.status {
                        match status.as_str() {
                            "StandBy" => return "Standby".to_string(),
                            "PoweredOn" => {
                                is_powered_on = true;
                                break;
                            }
                            _ => return status.clone(),
                        }
                    }
                }
            }
        }

        if !is_powered_on {
            return "Unknown".to_string();
        }

        // Machine is powered on, now check boiler status
        for widget in &self.widgets {
            if widget.code == "CMCoffeeBoiler" {
                if let Some(output) = &widget.output {
                    if let Some(status) = &output.status {
                        if status == "Ready" {
                            return "On (Ready)".to_string();
                        } else if let Some(ready_time) = output.ready_start_time {
                            let now = current_time_ms.unwrap_or_else(|| {
                                SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap()
                                    .as_millis() as u64
                            });

                            if ready_time > now {
                                let seconds_remaining = (ready_time - now) / 1000;
                                let minutes_remaining = seconds_remaining / 60;

                                if minutes_remaining == 0 {
                                    return "On (Ready in < 1 min)".to_string();
                                } else if minutes_remaining == 1 {
                                    return "On (Ready in 1 min)".to_string();
                                } else {
                                    return format!("On (Ready in {} mins)", minutes_remaining);
                                }
                            } else {
                                // Ready time is in the past, should be ready soon
                                return "On (Ready in < 1 min)".to_string();
                            }
                        } else {
                            // Heating but no ready time
                            return "On (Ready soon)".to_string();
                        }
                    }
                }
            }
        }

        // Machine is on but we don't have boiler info
        "On".to_string()
    }
}

impl Machine {
    pub async fn get_status_display(&self, client: &crate::client::LaMarzoccoClient) -> String {
        if !self.connected {
            return "Unavailable".to_string();
        }

        match client.get_machine_status(&self.serial_number).await {
            Ok(status) => status.get_status_string(),
            Err(_) => "Unknown".to_string(),
        }
    }
}

impl MachineCommand {
    pub fn turn_on() -> Self {
        Self {
            mode: "BrewingMode".to_string(),
        }
    }

    pub fn turn_off() -> Self {
        Self {
            mode: "StandBy".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_machine_command_creation() {
        let on_command = MachineCommand::turn_on();
        assert_eq!(on_command.mode, "BrewingMode");

        let off_command = MachineCommand::turn_off();
        assert_eq!(off_command.mode, "StandBy");
    }

    #[test]
    fn test_machine_command_json_serialization() {
        let on_command = MachineCommand::turn_on();
        let json = serde_json::to_string(&on_command).unwrap();
        assert!(json.contains("BrewingMode"));

        let off_command = MachineCommand::turn_off();
        let json = serde_json::to_string(&off_command).unwrap();
        assert!(json.contains("StandBy"));
    }

    #[test]
    fn test_machine_status_parsing() {
        // Test StandBy status
        let standby_status = MachineStatus {
            widgets: vec![Widget {
                code: "CMMachineStatus".to_string(),
                output: Some(WidgetOutput {
                    status: Some("StandBy".to_string()),
                    mode: None,
                    ready_start_time: None,
                }),
            }],
        };

        assert!(!standby_status.is_on());
        assert_eq!(standby_status.get_status_string(), "Standby");

        // Test PoweredOn status
        let powered_on_status = MachineStatus {
            widgets: vec![Widget {
                code: "CMMachineStatus".to_string(),
                output: Some(WidgetOutput {
                    status: Some("PoweredOn".to_string()),
                    mode: None,
                    ready_start_time: None,
                }),
            }],
        };

        assert!(powered_on_status.is_on());
        assert_eq!(powered_on_status.get_status_string(), "On");

        // Test missing widget
        let no_status = MachineStatus { widgets: vec![] };

        assert!(!no_status.is_on());
        assert_eq!(no_status.get_status_string(), "Unknown");
    }

    #[test]
    fn test_machine_status_with_ready_time() {
        // Test warming status with ready_start_time
        let status_warming = MachineStatus {
            widgets: vec![
                Widget {
                    code: "CMMachineStatus".to_string(),
                    output: Some(WidgetOutput {
                        status: Some("PoweredOn".to_string()),
                        mode: None,
                        ready_start_time: None, // This widget doesn't have ready time
                    }),
                },
                Widget {
                    code: "CMCoffeeBoiler".to_string(),
                    output: Some(WidgetOutput {
                        status: Some("Heating".to_string()),
                        mode: None,
                        ready_start_time: Some(1748515947000), // Future timestamp
                    }),
                },
            ],
        };

        assert!(status_warming.is_on());

        // Test with a fixed current time to avoid flaky tests
        // readyStartTime in fixture is 1748515947000 (Jan 29, 2025 15:32:27 UTC)
        // Let's set current time to 5 minutes earlier: 1748515647000
        let fixed_current_time = 1748515647000; // 5 minutes before ready time
        let warming_status = status_warming.get_status_string_with_time(Some(fixed_current_time));
        assert_eq!(warming_status, "On (Ready in 5 mins)");

        // Test when current time is exactly at ready time
        let warming_status_ready = status_warming.get_status_string_with_time(Some(1748515947000));
        assert_eq!(warming_status_ready, "On (Ready in < 1 min)");

        // Test when current time is past ready time
        let warming_status_past = status_warming.get_status_string_with_time(Some(1748515947001));
        assert_eq!(warming_status_past, "On (Ready in < 1 min)");

        // Test when ready in 1 minute
        let one_minute_before = 1748515947000 - 60000; // 1 minute before
        let warming_status_1min =
            status_warming.get_status_string_with_time(Some(one_minute_before));
        assert_eq!(warming_status_1min, "On (Ready in 1 min)");

        // Test when ready in less than 1 minute
        let thirty_seconds_before = 1748515947000 - 30000; // 30 seconds before
        let warming_status_soon =
            status_warming.get_status_string_with_time(Some(thirty_seconds_before));
        assert_eq!(warming_status_soon, "On (Ready in < 1 min)");
    }

    #[test]
    fn test_machine_status_error_conditions() {
        // Test empty widgets
        let status = MachineStatus { widgets: vec![] };
        assert!(!status.is_on());
        assert_eq!(status.get_status_string(), "Unknown");

        // Test wrong widget type
        let status = MachineStatus {
            widgets: vec![Widget {
                code: "WrongWidget".to_string(),
                output: Some(WidgetOutput {
                    status: Some("PoweredOn".to_string()),
                    mode: None,
                    ready_start_time: None,
                }),
            }],
        };
        assert!(!status.is_on());
        assert_eq!(status.get_status_string(), "Unknown");

        // Test no output
        let status = MachineStatus {
            widgets: vec![Widget {
                code: "CMMachineStatus".to_string(),
                output: None,
            }],
        };
        assert!(!status.is_on());
        assert_eq!(status.get_status_string(), "Unknown");

        // Test no status field
        let status = MachineStatus {
            widgets: vec![Widget {
                code: "CMMachineStatus".to_string(),
                output: Some(WidgetOutput {
                    status: None,
                    mode: Some("SomeMode".to_string()),
                    ready_start_time: None,
                }),
            }],
        };
        assert!(!status.is_on());
        assert_eq!(status.get_status_string(), "Unknown");
    }

    #[test]
    fn test_machine_properties() {
        let machine = Machine {
            serial_number: "TEST123".to_string(),
            model: Some("Test Model".to_string()),
            name: Some("Test Machine".to_string()),
            location: Some("Test Location".to_string()),
            connected: false,
        };

        // Test machine properties
        assert_eq!(machine.serial_number, "TEST123");
        assert_eq!(machine.model, Some("Test Model".to_string()));
        assert_eq!(machine.name, Some("Test Machine".to_string()));
        assert_eq!(machine.location, Some("Test Location".to_string()));
        assert!(!machine.connected);
    }
}

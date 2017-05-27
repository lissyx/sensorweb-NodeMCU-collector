use std::collections::HashMap;

#[derive(Debug)]
pub struct NetworkMsg {
    host: String,
    time: f32,
    msg:  MessageContent
}

#[derive(Debug, Eq, PartialEq)]
struct MessageContent {
    mtype: MessageType,
    mvals: HashMap<String, String>
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum MessageType {
    UnknownMessage,
    NodeUp,
    Ntp,
    Loop,
    NtpSync,
    Session,
    AirCasting,
}

pub fn parse_from_string(s: String) -> NetworkMsg {
    let empty_rv = NetworkMsg {
        host: String::from(""),
        time: 0.0,
        msg:  parse_msg_content("")
    };

    let mut elements = s.splitn(2, ":");
    if let Some(msg_host) = elements.next() {
        debug!("msg_host={}", msg_host);
        let msg_time = {
            if let Some(next_element) = elements.next() {
                let start_time   = next_element.find("[");
                debug!("start_time={:?}", start_time);
                let stop_time    = next_element.find("]");
                debug!("stop_time={:?}", stop_time);
                if let Some(start_time_idx) = start_time {
                    if let Some(stop_time_idx) = stop_time {
                        let substr = &next_element[start_time_idx+1..stop_time_idx];
                        let time_as_f32 = substr.parse::<f32>().unwrap();
                        debug!("extract: start_time={:?} stop_time={:?}", start_time, stop_time);
                        debug!("extract: substr={}", time_as_f32);
                        time_as_f32
                    } else {
                        -1.0
                    }
                } else {
                    -1.0
                }
            } else {
                -1.0
            }
        };

        let mut end_payload = s.splitn(3, " ");
        let msg_msg = if let Some(end_str) = end_payload.last() {
                parse_msg_content(end_str)
            } else {
                parse_msg_content("")
            };

        return NetworkMsg {
            host: msg_host.into(),
            time: msg_time,
            msg:  msg_msg.into()
        };
    }

    empty_rv
}

fn parse_msg_content(s: &str) -> MessageContent {
    let default_rv = MessageContent {
        mtype: MessageType::UnknownMessage,
        mvals: HashMap::new()
    };

    let mut elements = s.splitn(2, ":");
    if let Some(identifier) = elements.next() {
        let msg_identifier = match identifier.to_lowercase().as_ref() {
            "up"      => MessageType::NodeUp,
            "ntp"     => MessageType::Ntp,
            "loop"    => MessageType::Loop,
            "ntpsyncevent" => MessageType::NtpSync,
            "sessionuuid"  => MessageType::Session,
            "ac"      => MessageType::AirCasting,
            _         => MessageType::UnknownMessage
        };

        let mut hashmap = if let Some(end_str) = elements.last() {
            parse_to_hashmap(msg_identifier, end_str)
        } else {
            HashMap::new()
        };

        MessageContent {
            mtype: msg_identifier,
            mvals: hashmap
        }
    } else {
        default_rv
    }
}

fn parse_to_hashmap(msg_identifier: MessageType, end_str: &str) -> HashMap<String, String> {
    let mut hashmap = HashMap::new();
    let end_str_clean = end_str.trim();
    debug!("end of string: {:?}", end_str_clean);

    match msg_identifier {
        MessageType::NodeUp         => parse_nodeup(end_str_clean, &mut hashmap),
        MessageType::Ntp            => parse_ntp(end_str_clean, &mut hashmap),
        MessageType::Loop           => parse_loop(end_str_clean, &mut hashmap),
        MessageType::NtpSync        => parse_ntpsync(end_str_clean, &mut hashmap),
        MessageType::Session        => parse_session(end_str_clean, &mut hashmap),
        MessageType::AirCasting     => parse_aircasting(end_str_clean, &mut hashmap),
        MessageType::UnknownMessage => {}
    }

    hashmap
}

fn parse_nodeup(s: &str, h: &mut HashMap<String, String>) {
    // 1.0:May 14 2017 01:34:24@192.168.1.29"
    let mut elts = s.splitn(2, ":");
    let version_str = if let Some(a) = elts.next() {
        a
    } else {
        "0.0.0"
    };
    debug!("read version_str={}", version_str);
    h.insert(String::from("version"), String::from(version_str));

    if let Some(next_end_str) = elts.last() {
        let mut elts2 = next_end_str.splitn(2, "@");
        let builddate = if let Some(a) = elts2.next() {
            a
        } else {
            "N/A"
        };
        let ip_addr = if let Some(a) = elts2.next() {
            a
        } else {
            "N/A"
        };

        debug!("read builddate={}", builddate);
        h.insert(String::from("builddate"), String::from(builddate));
        debug!("read ip_addr={}", ip_addr);
        h.insert(String::from("ip_addr"), String::from(ip_addr));
    }
}

fn parse_aircasting(s: &str, h: &mut HashMap<String, String>) {
    // push: Code 200
    let mut split_cmd = s.splitn(2, ":");
    if let Some(ac_cmd) = split_cmd.next() {
        let cmd_loc = ac_cmd.trim().to_lowercase();
        let cmd_ref = cmd_loc.as_ref();
        debug!("read cmd_ref={}", cmd_ref);
        h.insert(String::from("command"), String::from(cmd_ref));
        match cmd_ref {
            "push" => {
                if let Some(next_split_cmd) = split_cmd.last() {
                    let split_code = next_split_cmd.trim().splitn(2, " ");
                    if let Some(code_value) = split_code.last() {
                        debug!("read code_value={}", code_value);
                        h.insert(String::from("http_code"), String::from(code_value));
                    }
                }
            },
            _      => {}
        }
    }
}

fn parse_ntp(s: &str, h: &mut HashMap<String, String>) {
    if s.find("PM2.5").is_some() {
        // 2017-05-26T15:27:53.000+01:00 PM2.5: 12 UUID:d687fe3f-2d30-352d-0c21-ff3f2cea2040 sent:1
        let mut split_ntp = s.splitn(5, " ");

        if let Some(date) = split_ntp.next() {
            let date_iso8601 = date.trim();
            debug!("read date_iso8601={}", date_iso8601);
            h.insert(String::from("datetime"), String::from(date_iso8601));
        }

        if let Some(pm25_head) = split_ntp.next() {
            if let Some(pm25_val) = split_ntp.next() {
                debug!("read pm25_val={}", pm25_val);
                h.insert(String::from("pm2.5"), String::from(pm25_val));
            }
        }

        if let Some(uuid) = split_ntp.next() {
            let mut uuid_all = uuid.splitn(2, ":");
            let _ = uuid_all.next();
            if let Some(uuid_val) = uuid_all.next() {
                debug!("read uuid_val={}", uuid_val);
                h.insert(String::from("UUID"), String::from(uuid_val));
            }
        }

        if let Some(sent) = split_ntp.next() {
            let mut sent_all = sent.splitn(2, ":");
            let _ = sent_all.next();
            if let Some(sent_val) = sent_all.next() {
                debug!("read sent_val={}", sent_val);
                h.insert(String::from("sent"), String::from(sent_val));
            }
        }
    }
}

fn parse_loop(s: &str, h: &mut HashMap<String, String>) {
    // Loop: deepSleep: nextInterval=288.93; executionTime=11.07 ; slowDownFactor=1.04365; deepSleep(301.54)
    let mut split_action = s.splitn(2, " ");
    if let Some(ac_action) = split_action.next() {
        let action_loc = ac_action.trim().to_lowercase().replace(":", "");
        let action_ref = action_loc.as_ref();
        match action_ref {
            "deepsleep" => {
                if let Some(next_split_action) = split_action.last() {
                    let mut split_state = next_split_action.trim().splitn(5, ";");
                    let _ = split_state.next(); // nextInterval
                    let _ = split_state.next(); // executionTime

                    if let Some(slow_down_str) = split_state.next() {
                        let slow_down_str_split = slow_down_str.splitn(2, "=");
                        if let Some(slow_down) = slow_down_str_split.last() {
                            debug!("read slow_down={}", slow_down);
                            h.insert(String::from("slowdownfactor"), String::from(slow_down));
                        }
                    }

                    if let Some(deep_sleep_str) = split_state.next() {
                        let deep_sleep_str_split = deep_sleep_str.splitn(2, "(");
                        if let Some(deep_sleep_str_left) = deep_sleep_str_split.last() {
                            let mut deep_sleep_str_split_right = deep_sleep_str_left.splitn(2, ")");
                            if let Some(deep_sleep) = deep_sleep_str_split_right.next() {
                                debug!("read deep_sleep={}", deep_sleep);
                                h.insert(String::from("deepsleepduration"), String::from(deep_sleep));
                            }
                        }
                    }
                }

                debug!("read action_ref={}", action_ref);
                h.insert(String::from("action"), String::from(action_ref));
            },
            "no" => {
                if let Some(next_split_action) = split_action.last() {
                    let mut split_state = next_split_action.trim().split(" ");
                    for e in split_state {
                        if e.find("sleepWakeCycles=").is_some() {
                            let sleepwake_str_split = e.splitn(2, "=");
                            if let Some(sleepwake) = sleepwake_str_split.last() {
                                debug!("read sleepwake={}", sleepwake);
                                h.insert(String::from("sleepwakecycles"), String::from(sleepwake));
                            }
                        } else if e.find("ntpErrors=").is_some() {
                            let ntperr_str_split = e.splitn(2, "=");
                            if let Some(ntperr) = ntperr_str_split.last() {
                                debug!("read ntperr={}", ntperr);
                                h.insert(String::from("ntperrors"), String::from(ntperr));
                            }
                        }
                    }
                }

                debug!("read action_ref={}", action_ref);
                h.insert(String::from("action"), String::from("waitntp"));
            },
            _      => {}
        }
    }
}

fn parse_ntpsync(s: &str, h: &mut HashMap<String, String>) {
    // We want no " -- " AND no " => "
    if s.find(" -- ").is_none() && s.find(" => ").is_none() {
        let ntpdate = s.trim();
        debug!("read ntpdate={}", ntpdate);
        h.insert(String::from("ntpdate"), String::from(ntpdate));
    }
}

fn parse_session(s: &str, h: &mut HashMap<String, String>) {
    if s.find("-").is_some() {
        let uuid = s.trim();
        debug!("read uuid={}", uuid);
        h.insert(String::from("UUID"), String::from(uuid));
    }
}

#[test]
fn test_parse_msgs() {
    let msg0 = String::from("");
    let n0 = parse_from_string(msg0);
    assert_eq!(n0.host, String::from(""));
    assert_eq!(n0.time, -1.0);
    assert_eq!(n0.msg.mtype, MessageType::UnknownMessage);
    assert!(n0.msg.mvals.is_empty());

    let msg1 = String::from("ESP_D427A9: [2.89900] UP: 1.0:May 14 2017 01:34:24@192.168.1.29");
    let n1 = parse_from_string(msg1);
    assert_eq!(n1.host, String::from("ESP_D427A9"));
    assert_eq!(n1.time, 2.899);
    assert_eq!(n1.msg.mtype, MessageType::NodeUp);
    assert_eq!(n1.msg.mvals[&String::from("version")], String::from("1.0"));
    assert_eq!(n1.msg.mvals[&String::from("builddate")], String::from("May 14 2017 01:34:24"));
    assert_eq!(n1.msg.mvals[&String::from("ip_addr")], String::from("192.168.1.29"));

    let msg2 = String::from("ESP_D427A9: [11.06000] AC:push: Code 200");
    let n2 = parse_from_string(msg2);
    assert_eq!(n2.host, String::from("ESP_D427A9"));
    assert_eq!(n2.time, 11.06);
    assert_eq!(n2.msg.mtype, MessageType::AirCasting);
    assert_eq!(n2.msg.mvals[&String::from("command")], String::from("push"));
    assert_eq!(n2.msg.mvals[&String::from("http_code")], String::from("200"));

    let msg3 = String::from("ESP_D427A9: [11.06500] NTP: 2017-05-26T15:27:53.000+01:00 PM2.5: 12 UUID:d687fe3f-2d30-352d-0c21-ff3f2cea2040 sent:1");
    let n3 = parse_from_string(msg3);
    assert_eq!(n3.host, String::from("ESP_D427A9"));
    assert_eq!(n3.time, 11.065);
    assert_eq!(n3.msg.mtype, MessageType::Ntp);
    assert_eq!(n3.msg.mvals[&String::from("datetime")], String::from("2017-05-26T15:27:53.000+01:00"));
    assert_eq!(n3.msg.mvals[&String::from("pm2.5")], String::from("12"));
    assert_eq!(n3.msg.mvals[&String::from("UUID")], String::from("d687fe3f-2d30-352d-0c21-ff3f2cea2040"));
    assert_eq!(n3.msg.mvals[&String::from("sent")], String::from("1"));

    let msg4 = String::from("ESP_D427A9: [11.15300] Loop: deepSleep: nextInterval=288.93; executionTime=11.07 ; slowDownFactor=1.04365; deepSleep(301.54)");
    let n4 = parse_from_string(msg4);
    assert_eq!(n4.host, String::from("ESP_D427A9"));
    assert_eq!(n4.time, 11.153);
    assert_eq!(n4.msg.mtype, MessageType::Loop);
    assert_eq!(n4.msg.mvals[&String::from("action")], String::from("deepsleep"));
    assert_eq!(n4.msg.mvals[&String::from("slowdownfactor")], String::from("1.04365"));
    assert_eq!(n4.msg.mvals[&String::from("deepsleepduration")], String::from("301.54"));

    let msg5 = String::from("ESP_D427A9: [7.97900] NTPSyncEvent: 16:24:59 30/05/2017");
    let n5 = parse_from_string(msg5);
    assert_eq!(n5.host, String::from("ESP_D427A9"));
    assert_eq!(n5.time, 7.979);
    assert_eq!(n5.msg.mtype, MessageType::NtpSync);
    assert_eq!(n5.msg.mvals[&String::from("ntpdate")], String::from("16:24:59 30/05/2017"));

    let msg6 = String::from("ESP_D427A9: [8.14600] SessionUUID: d687fe3f-2d30-352d-0c21-ff3f2cea2040");
    let n6 = parse_from_string(msg6);
    assert_eq!(n6.host, String::from("ESP_D427A9"));
    assert_eq!(n6.time, 8.146);
    assert_eq!(n6.msg.mtype, MessageType::Session);
    assert_eq!(n6.msg.mvals[&String::from("UUID")], String::from("d687fe3f-2d30-352d-0c21-ff3f2cea2040"));

    let msg7 = String::from("ESP_D427A9: [4.07500] Loop: no NTP initial sync, waiting ... sleepWakeCycles=1 ntpErrors=2");
    let n7 = parse_from_string(msg7);
    assert_eq!(n7.host, String::from("ESP_D427A9"));
    assert_eq!(n7.time, 4.075);
    assert_eq!(n7.msg.mtype, MessageType::Loop);
    assert_eq!(n7.msg.mvals[&String::from("action")], String::from("waitntp"));
    assert_eq!(n7.msg.mvals[&String::from("sleepwakecycles")], String::from("1"));
    assert_eq!(n7.msg.mvals[&String::from("ntperrors")], String::from("2"));
}

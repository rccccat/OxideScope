use chrono::Local;
use regex::Regex;
use std::net::IpAddr;

pub fn now_string() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}

pub fn expand_targets(raw: &str, ignore: &str) -> Vec<String> {
    let (ignore_list, regex_list) = generate_ignore(ignore);
    let mut seen = std::collections::BTreeSet::new();
    for line in raw.lines() {
        let t = line.trim();
        if t.is_empty() { continue; }
        let items = generate_target(t);
        for r in items {
            if r.trim().is_empty() { continue; }
            if ignore_list.contains(&r) { continue; }
            if !regex_list.is_empty() {
                let mut ok = true;
                for re in &regex_list {
                    if re.is_match(&r) == false { ok = false; break; }
                }
                if ok { seen.insert(r); }
            } else {
                seen.insert(r);
            }
        }
    }
    seen.into_iter().collect()
}

fn generate_target(target: &str) -> Vec<String> {
    // very light expansion: CIDR ranges and single targets
    if target.contains("://") {
        return vec![target.to_string()];
    }
    if let Some((start, end)) = target.split_once('-') {
        if let (Ok(a), Ok(b)) = (start.parse::<IpAddr>(), end.parse::<IpAddr>()) {
            if let (IpAddr::V4(sa), IpAddr::V4(sb)) = (a, b) {
                let mut res = vec![];
                let mut cur = u32::from(sa);
                let endn = u32::from(sb);
                while cur <= endn {
                    res.push(std::net::Ipv4Addr::from(cur).to_string());
                    if cur == endn { break; }
                    cur = cur.saturating_add(1);
                }
                return res;
            }
        }
    }
    if target.contains('/') {
        if let Ok(net) = target.parse::<ipnet::IpNet>() {
            let mut out = vec![];
            match net {
                ipnet::IpNet::V4(n) => {
                    for ip in n.hosts() { out.push(ip.to_string()); }
                }
                ipnet::IpNet::V6(_) => {}
            }
            return out;
        }
    }
    vec![target.to_string()]
}

fn generate_ignore(ignore: &str) -> (std::collections::BTreeSet<String>, Vec<Regex>) {
    let mut ignore_set = std::collections::BTreeSet::new();
    let mut regexes = vec![];
    for line in ignore.lines() {
        let mut t = line.replace("http://", "").replace("https://", "");
        t = t.trim().to_string();
        if t.is_empty() { continue; }
        if t.contains('*') {
            let esc = regex::escape(&t).replace(r"\*", ".*");
            if let Ok(re) = Regex::new(&esc) { regexes.push(re); }
        } else {
            ignore_set.insert(t);
        }
    }
    (ignore_set, regexes)
}

use std::str::FromStr;
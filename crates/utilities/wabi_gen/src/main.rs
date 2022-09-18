use serde_json::{Map, Value};
use xshell::{cmd, Shell};

fn main() {
    let sh = Shell::new().unwrap();

    let packages = cmd!(sh, "cargo doc --package bevy --message-format json")
        .read()
        .unwrap()
        .lines()
        .map(|l| {
            serde_json::from_str::<Map<String, Value>>(l).unwrap()
        })
        .collect::<Vec<_>>();
    
    for package in packages {
        println!("{:?}", package.keys().collect::<Vec<_>>());
    }
}

// use duct::cmd;

// pub struct WifiCondition {
//     wifi_name: String,
// }

// // impl WifiCondition {
// //     pub fn new(wifi_name: &str) -> Self {
// //         WifiCondition {
// //             wifi_name: wifi_name.to_string(),
// //         }
// //     }
// // }

// pub fn sync_condition(wifi_name: &str) -> bool {
//     let output = cmd("iwgetid", "-r").read().expect("Failed to execute iwgetid");

//     if output.status.success() {
//         let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
//         ssid == wifi_name
//     } else {
//         false
//     }
// }

// pub async fn async_condition(wifi_name: &str) -> bool {
//     let output = cmd("iwgetid", "-r").read().await.expect("Failed to execute iwgetid");

//     if output.status.success() {
//         let ssid = String::from_utf8_lossy(&output.stdout).trim().to_string();
//         ssid == wifi_name
//     } else {
//         false
//     }
// }

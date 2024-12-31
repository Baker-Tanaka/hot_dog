use dioxus::logger::tracing::{error, info};
use dioxus::prelude::*;
use probe_rs::flashing::{download_file, Format};
use probe_rs::probe::list::Lister;
use probe_rs::rtt::Rtt;
use probe_rs::Permissions;
use rfd::FileDialog;
use std::fs::File;
use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;

enum Command {
    Start(String),
    Stop,
}

static CMD: Mutex<Option<Command>> = Mutex::new(None);

fn write_and_print_rtt(cmd: Command) -> anyhow::Result<()> {
    match cmd {
        Command::Start(firmware_path) => {
            let lister = Lister::new();
            let probes = lister.list_all();

            // Open the probe
            let probe = probes[0].open()?;

            // Start the session
            let mut session = probe.attach("rp2040", Permissions::default())?;

            // Write the firmware
            download_file(&mut session, firmware_path, Format::Elf)?;

            let mut core = session.core(0)?;

            let mut rtt = Rtt::attach(&mut core)?;

            core.reset()?;
            core.run()?;
            for _ in 0..10 {
                if let Some(input) = rtt.up_channel(0) {
                    let mut buf = [0u8; 1024];
                    let count = input.read(&mut core, &mut buf[..])?;
                    info!("Read data: {:?}", std::str::from_utf8(&buf[..count])?);
                }
                std::thread::sleep(std::time::Duration::from_secs(5));
            }

            core.halt(Duration::from_secs(5))?;
        }
        Command::Stop => {}
    };
    Ok(())
}

fn main() {
    dioxus::logger::init(dioxus::logger::tracing::Level::INFO).expect("Failed to init logger");
    dioxus::launch(App);
}

struct UploadFile {
    name: String,
    checksum: String,
    file_path: String,
}

#[component]
fn App() -> Element {
    let mut file_uploaded = use_signal(|| None as Option<UploadFile>);
    let mut automatilly_write = use_signal(|| false);
    let mut write_and_test = use_signal(|| 0);

    let select_file = move |_| {
        if let Some(file) = FileDialog::new().pick_file() {
            let file_name = file
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            let firmware_path = file.to_string_lossy().to_string();
            let mut file_content = Vec::new();
            let mut file = File::open(&file).expect("ファイルを開けませんでした");
            file.read_to_end(&mut file_content)
                .expect("ファイルの読み込みに失敗しました");
            let checksum = chksum::sha2_256::chksum(&file_content).unwrap();
            file_uploaded.set(Some(UploadFile {
                name: file_name,
                checksum: checksum.to_string(),
                file_path: firmware_path.clone(),
            }));
        }
    };

    use_future(move || async move {
        loop {
            if let Some(cmd) = CMD.lock().unwrap().take() {
                match write_and_print_rtt(cmd) {
                    Ok(_) => {
                        write_and_test += 1;
                    }
                    Err(e) => {
                        error!("{:?}", e);
                    }
                }
            };
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    rsx! {
        div {
            class: "container",
            h1 { "自動Baker Link. Dev書き込み君" }

            div{
                button{
                    onclick: move |_| {
                        if let Some(file) = &*file_uploaded.read() {
                            let mut cmd = CMD.lock().unwrap();
                            *cmd = Some(Command::Start(file.file_path.clone()));
                        }
                    },
                    "スタート"
                }
                button {
                    onclick: select_file,
                    "ファイル選択"
                }
            }

            div {
                class: "file-details",
                p{ "書き込み＆テスト回数: {write_and_test}" }
                if let Some(file) = &*file_uploaded.read() {
                    p { "ファイル名: {file.name}" }
                    pre { "CheckSum: {file.checksum}" }
                } else {
                    p { "ファイルが選択されていません。" }
                }
            }
        }
    }
}

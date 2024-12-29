use dioxus::prelude::*;
use dioxus_elements::{button, FileEngine};
use std::sync::Arc;

fn main() {
    dioxus::launch(App);
}

struct UploadFile {
    name: String,
    contents: String,
}

#[component]
fn App() -> Element {
    let mut file_uploaded = use_signal(|| None as Option<UploadFile>);
    let mut automatilly_write = use_signal(|| false);
    let mut write_and_test = use_signal(|| 0);

    let read_files = move |file_engine: Arc<dyn FileEngine>| async move {
        let files = file_engine.files();
        for file_name in &files {
            if let Some(contents) = file_engine.read_file_to_string(file_name).await {
                file_uploaded.set(Some(UploadFile {
                    name: file_name.clone(),
                    contents,
                }));
            }
        }
    };

    let upload_files = move |evt: FormEvent| async move {
        if let Some(file_engine) = evt.files() {
            read_files(file_engine).await;
        }
    };

    rsx! {
        div {
            class: "container",
            h1 { "自動Baker Link. Dev書き込み君" }

            div {
                class: "file-upload",
                label { r#for: "fileInput", ".elfを選んでね" }
                input {
                    r#type: "file",
                    id: "fileInput",
                    onchange: upload_files,
                }
            }

            div{
                button{
                    onclick: move |_| automatilly_write.set(true),
                    "スタート"
                }
                button {
                    onclick: move |_| automatilly_write.set(false),
                    "ストップ"
                }
            }

            div {
                class: "file-details",
                p{ "書き込み＆テスト回数: {write_and_test}" }
                if let Some(file) = &*file_uploaded.read() {
                    p { "ファイル名: {file.name}" }
                    pre { "{file.contents}" }
                } else {
                    p { "ファイルが選択されていません。" }
                }
            }
        }
    }
}

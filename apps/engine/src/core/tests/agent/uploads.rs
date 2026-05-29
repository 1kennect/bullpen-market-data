//! Files a caller attaches: copied into the sandbox workspace and named in the prompt.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::core::tests::support::{MemorySink, Scripted, ctx, text};
use crate::core::traits::tools::files::FileSource;
use crate::core::traits::tools::sandbox::Sandbox;
use crate::core::types::agent::AgentConfig;
use crate::core::types::agent::context::RequestContext;
use crate::core::types::conversation::file::{FileAttachment, FileError};
use crate::core::types::tools::sandbox::{
    SandboxCommand, SandboxError, SandboxOutput, SandboxRequest, SandboxSession,
};
use crate::core::types::trace::TraceEvent;
use crate::runtime::harness::agent::{Agent, AgentDeps};
use crate::runtime::harness::safety::policy::RiskPolicy;
use crate::runtime::harness::tools::ToolSet;
use crate::runtime::tools::files::HttpFiles;

/// A file source holding fixed bytes for every link, or refusing every link.
struct Served(Result<Vec<u8>, String>);

#[async_trait]
impl FileSource for Served {
    async fn fetch(
        &self,
        _ctx: &RequestContext,
        _file: &FileAttachment,
    ) -> Result<Vec<u8>, FileError> {
        self.0.clone().map_err(FileError::Fetch)
    }
}

/// A sandbox that records what is written to it and answers every command with converted.
#[derive(Default)]
struct Workspace {
    written: Mutex<Vec<(String, String, usize)>>,
    ran: Mutex<Vec<String>>,
    converted: Option<String>,
}

impl Workspace {
    fn converting(stdout: &str) -> Self {
        Self {
            converted: Some(stdout.to_owned()),
            ..Self::default()
        }
    }
}

#[async_trait]
impl Sandbox for Workspace {
    async fn run(
        &self,
        _ctx: &RequestContext,
        request: &SandboxRequest,
    ) -> Result<SandboxOutput, SandboxError> {
        if let Ok(mut ran) = self.ran.lock() {
            ran.push(request.command.clone());
        }
        match &self.converted {
            Some(stdout) => Ok(SandboxOutput {
                exit_code: 0,
                stdout: stdout.clone(),
                stderr: String::new(),
                session: request.session.clone(),
            }),
            None => Err(SandboxError::Refused("not in this test".into())),
        }
    }
    fn enabled(&self) -> bool {
        true
    }
    fn set_enabled(&self, _on: bool) {}
    fn sessions(&self) -> Vec<SandboxSession> {
        Vec::new()
    }
    fn commands(&self) -> Vec<SandboxCommand> {
        Vec::new()
    }
    async fn kill(&self, _name: &str) -> Result<(), SandboxError> {
        Ok(())
    }
    async fn put(
        &self,
        _ctx: &RequestContext,
        session: &str,
        name: &str,
        content: &[u8],
    ) -> Result<String, SandboxError> {
        if let Ok(mut written) = self.written.lock() {
            written.push((session.to_owned(), name.to_owned(), content.len()));
        }
        Ok(format!("/tmp/{name}"))
    }
}

fn report() -> FileAttachment {
    FileAttachment {
        url: "https://cdn.discordapp.com/attachments/1/2/report.pdf".into(),
        name: "Q3 earnings report.pdf".into(),
        media_type: "application/pdf".into(),
        size: 1_200,
    }
}

fn agent_with(
    model: Scripted,
    files: Served,
    workspace: Arc<Workspace>,
) -> (Agent, Arc<MemorySink>) {
    let sink = Arc::new(MemorySink::new());
    let deps = AgentDeps {
        model: Arc::new(model),
        tools: ToolSet::new(),
        policy: Arc::new(RiskPolicy::default()),
        trace: sink.clone(),
        conversations: None,
        memory: None,
        confirmations: None,
        compactor: None,
        guardrail: None,
        profile: None,
        profile_graph: None,
        sandbox: Some(workspace),
        files: Some(Arc::new(files)),
    };
    (Agent::new(deps, AgentConfig::default(), "sys"), sink)
}

/// Every message the model was sent on its first call, joined.
fn first_prompt(model: &Scripted) -> Arc<Mutex<Vec<crate::core::types::model::ModelRequest>>> {
    model.sent()
}

#[tokio::test]
async fn an_attached_file_lands_in_the_workspace_and_its_text_reaches_the_prompt() {
    let model = Scripted::new(vec![Ok(text("Revenue grew 12 percent."))]);
    let sent = first_prompt(&model);
    let workspace = Arc::new(Workspace::converting(
        "CHARS 4200\n=== PREVIEW\nQ3 Earnings Report\n=== MATCHES\n[line 40]\nRevenue: up 12 percent",
    ));
    let (agent, sink) = agent_with(model, Served(Ok(vec![b'%'; 1_200])), workspace.clone());
    let ctx = ctx().with_files(vec![report()]);
    assert!(agent.run(&ctx, "how much did revenue grow?").await.is_ok());

    let written = workspace
        .written
        .lock()
        .map(|w| w.clone())
        .unwrap_or_default();
    assert_eq!(written.len(), 1, "{written:?}");
    let (session, name, bytes) = &written[0];
    assert_eq!(name, "upload-Q3_earnings_report.pdf");
    assert_eq!(*bytes, 1_200);

    let prompt: String = sent
        .lock()
        .ok()
        .and_then(|r| r.first().cloned())
        .map(|r| r.messages.iter().map(|m| m.content.clone()).collect())
        .unwrap_or_default();
    assert!(
        prompt.contains("/tmp/upload-Q3_earnings_report.pdf"),
        "{prompt}"
    );
    assert!(prompt.contains(&format!("session {session}")), "{prompt}");
    assert!(prompt.contains("Q3 Earnings Report"), "{prompt}");
    assert!(
        prompt.contains("[line 40]\nRevenue: up 12 percent"),
        "{prompt}"
    );
    assert!(prompt.contains("4200 characters"), "{prompt}");
    assert!(
        prompt.contains("/tmp/upload-Q3_earnings_report.pdf.txt"),
        "{prompt}"
    );
    let ran = workspace.ran.lock().map(|r| r.clone()).unwrap_or_default();
    assert!(
        ran.first().is_some_and(|c| c.contains("pdftotext")),
        "the file is turned into text in the sandbox: {ran:?}"
    );
    let at_input = sent
        .lock()
        .ok()
        .and_then(|r| r.first().cloned())
        .is_some_and(|r| {
            let n = r.messages.len();
            r.messages
                .get(n.saturating_sub(2))
                .is_some_and(|m| m.content.contains("Revenue: up"))
        });
    assert!(at_input, "the file sits just before the question");
    assert!(
        sink.records()
            .iter()
            .any(|r| matches!(&r.event, TraceEvent::FileAttached { error: None, .. }))
    );
}

#[tokio::test]
async fn a_file_no_text_came_out_of_is_left_for_the_model_to_open() {
    let model = Scripted::new(vec![Ok(text("It is an image."))]);
    let sent = first_prompt(&model);
    let workspace = Arc::new(Workspace::converting(
        "CHARS 0\n=== PREVIEW\n\n=== MATCHES\n",
    ));
    let (agent, _) = agent_with(model, Served(Ok(vec![0; 10])), workspace);
    let ctx = ctx().with_files(vec![report()]);
    assert!(agent.run(&ctx, "what is this").await.is_ok());
    let prompt: String = sent
        .lock()
        .ok()
        .and_then(|r| r.first().cloned())
        .map(|r| r.messages.iter().map(|m| m.content.clone()).collect())
        .unwrap_or_default();
    assert!(prompt.contains("no text could be pulled"), "{prompt}");
}

#[tokio::test]
async fn a_file_that_cannot_be_read_is_named_as_unopened() {
    let model = Scripted::new(vec![Ok(text("I could not open it."))]);
    let sent = first_prompt(&model);
    let workspace = Arc::new(Workspace::default());
    let (agent, _) = agent_with(
        model,
        Served(Err("connection refused".into())),
        workspace.clone(),
    );
    let ctx = ctx().with_files(vec![report()]);
    assert!(agent.run(&ctx, "summarise this").await.is_ok());
    assert!(
        workspace.written.lock().is_ok_and(|w| w.is_empty()),
        "nothing is written"
    );
    let prompt: String = sent
        .lock()
        .ok()
        .and_then(|r| r.first().cloned())
        .map(|r| r.messages.iter().map(|m| m.content.clone()).collect())
        .unwrap_or_default();
    assert!(prompt.contains("could not be opened"), "{prompt}");
    assert!(prompt.contains("connection refused"), "{prompt}");
}

#[test]
fn only_files_within_the_cap_are_accepted_and_names_cannot_leave_the_workspace() {
    let big = FileAttachment {
        size: 3_000_000,
        ..report()
    };
    let kept = FileAttachment::accepted(vec![big, report()], 4, 2_000_000);
    assert_eq!(kept, vec![report()]);
    let sneaky = FileAttachment {
        name: "../../etc/passwd".into(),
        ..report()
    };
    for name in [
        "../../etc/passwd",
        "a..pdf",
        ".hidden",
        "x".repeat(200).as_str(),
    ] {
        let file = FileAttachment {
            name: name.to_owned(),
            ..report()
        };
        let safe = format!("upload-{}", file.safe_name());
        assert!(
            crate::core::types::tools::sandbox::workspace_path(&safe).is_ok(),
            "{name} became {safe}, which the workspace refuses"
        );
    }
    assert!(!sneaky.safe_name().contains('/'));
}

#[tokio::test]
async fn a_link_off_the_allowed_hosts_is_never_fetched() {
    let Ok(files) = HttpFiles::new(vec!["cdn.discordapp.com".into()], 2_000_000) else {
        unreachable!("the client builds")
    };
    for link in [
        "https://169.254.169.254/latest/meta-data",
        "http://cdn.discordapp.com/a.pdf",
        "https://evil.example/cdn.discordapp.com/a.pdf",
    ] {
        let file = FileAttachment {
            url: link.into(),
            ..report()
        };
        let out = files.fetch(&ctx(), &file).await;
        assert!(matches!(out, Err(FileError::Host(_))), "{link}: {out:?}");
    }
}

/// Live check that every kind of document a user uploads comes out as text in the real image.
#[tokio::test]
#[ignore = "needs a container runtime"]
async fn every_document_kind_comes_out_as_text_in_the_real_sandbox() {
    use std::time::Duration;

    use crate::core::types::tools::sandbox::Limits;
    use crate::runtime::harness::agent::uploads::{converted, terms, to_text};
    use crate::runtime::tools::sandbox::ContainerSandbox;

    let s = ContainerSandbox::new(Limits {
        timeout: Duration::from_mins(1),
        ..Limits::default()
    });
    let ctx = RequestContext::new("g", "u", Duration::from_mins(3));
    let session = "uploadkinds";
    let make = r#"python3 - <<'PY'
import docx, openpyxl
from PIL import Image, ImageDraw
d = docx.Document(); d.add_paragraph("Intro"); d.add_paragraph("The earnings call is on October 14 at 5pm."); d.save("/tmp/upload-report.docx")
wb = openpyxl.Workbook(); ws = wb.active; ws.append(["symbol", "shares"]); ws.append(["AAPL", "40 shares"]); wb.save("/tmp/upload-rooms.xlsx")
open("/tmp/upload-grades.csv", "w").write("symbol,price\nNVDA,182\n")
img = Image.new("RGB", (900, 200), "white"); ImageDraw.Draw(img).text((20, 80), "EARNINGS DATE JUNE 30", fill="black")
img.resize((2700, 600)).save("/tmp/upload-scan.pdf")
PY
python3 -c 'from PIL import Image, ImageDraw
i = Image.new("RGB", (900, 200), "white"); ImageDraw.Draw(i).text((20, 80), "Dividend is paid Tuesday at 3pm", fill="black")
i.resize((2700, 600)).save("/tmp/hours.png")'
tesseract /tmp/hours.png /tmp/upload-hours pdf 2>/dev/null
echo made"#;
    let Ok(out) = s
        .run(
            &ctx,
            &SandboxRequest {
                command: make.into(),
                session: Some(session.into()),
            },
        )
        .await
    else {
        unreachable!("the runtime answered")
    };
    assert!(out.stdout.contains("made"), "{}", out.stderr);

    for (path, question, want, ocr) in [
        (
            "/tmp/upload-report.docx",
            "when is the earnings call",
            "October 14",
            false,
        ),
        (
            "/tmp/upload-rooms.xlsx",
            "how many AAPL shares",
            "40 shares",
            false,
        ),
        (
            "/tmp/upload-grades.csv",
            "what is the NVDA price",
            "NVDA,182",
            false,
        ),
        (
            "/tmp/upload-hours.pdf",
            "when is the dividend paid",
            "Tuesday at 3pm",
            false,
        ),
        ("/tmp/upload-scan.pdf", "earnings date", "JUNE 30", true),
    ] {
        let Ok(out) = s
            .run(
                &ctx,
                &SandboxRequest {
                    command: to_text(path, &terms(question), 800, 2_400, 3, "en"),
                    session: Some(session.into()),
                },
            )
            .await
        else {
            unreachable!("the runtime answered")
        };
        let (read_by_ocr, chars, preview, matches) = converted(&out.stdout);
        assert!(
            chars > 0,
            "{path}: no text. stdout {} stderr {}",
            out.stdout,
            out.stderr
        );
        assert_eq!(read_by_ocr, ocr, "{path}");
        let flat = |text: &str| text.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(
            flat(&preview).contains(want) || flat(&matches).contains(want),
            "{path}: {want} not found in preview {preview:?} or matches {matches:?}"
        );
    }
}

#[test]
fn a_scan_is_read_by_paddleocr_with_tesseract_behind_it() {
    use crate::runtime::harness::agent::uploads::{tesseract_lang, to_text};

    let script = to_text("/tmp/upload-scan.pdf", &[], 800, 2_400, 3, "ch");
    assert!(script.contains("paddle-ocr --lang ch"), "{script}");
    assert!(
        script.contains("tesseract \"$1\" - -l chi_sim+eng"),
        "{script}"
    );
    assert!(
        script.contains("*.png|*.jpg"),
        "an image file is read by OCR too"
    );
    assert_eq!(tesseract_lang("en"), "eng");
    assert_eq!(tesseract_lang("korean"), "eng");
}

//! Tài khoản Claude — dựng dòng lệnh, môi trường và bản vá `settings.json` từ
//! TRƯỜNG CẤU TRÚC của [`ClaudeAccountCfg`].
//!
//! 🔴 Hàm THUẦN: không đọc đĩa, không đọc đồng hồ, không chạy gì. Chỗ nào cần
//! đọc tệp (`env_file`, `settings.json`) thì GĐ 2/GĐ 3 làm — ở đây chỉ nhận dữ
//! liệu đã có. Nhờ vậy mọi hành vi của trang quản lý đo được bằng bài kiểm
//! không cần mở cửa sổ nào.
//!
//! Ngoại lệ DUY NHẤT, từ GĐ 2: [`probe`] đọc đĩa (thư mục, `projects`,
//! `env_file`, `settings.json`, sổ hạn mức) và trả về dữ liệu thô; [`checks`]
//! vẫn THUẦN trên dữ liệu ấy, nên 8 mục chẩn đoán chấm được bằng `Probe` dựng
//! tay mà không cần chạm `$HOME`.
//!
//! **Luật 4 đứng trên cả tệp này:** cấu hình chỉ giữ ĐƯỜNG DẪN và TÊN biến.
//! Dòng lệnh dựng ra nạp giá trị bằng `$(cat …)` lúc CHẠY, nên giá trị không
//! bao giờ nằm trong `huba.config.json`, trong log, hay trong argv của huba.
//!
//! **`launch` legacy THẮNG** trong lúc di trú (kế hoạch GĐ 5): một tài khoản có
//! cả `launch` viết tay lẫn trường cấu trúc thì chạy đúng dòng viết tay ấy, và
//! [`effective_launch`] kêu lên để chủ máy biết trường cấu trúc đang bị che.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::config::{expand_home, home_dir, ClaudeAccountCfg, Config};
use crate::quota::Quota;
use crate::sessions::shell_quote;

/// Các trường CẤU TRÚC mà hàng này khai (khác rỗng), theo thứ tự cố định.
/// `config_dir` không tính: nó có từ trước và một mình nó không phải "dòng
/// lệnh dựng" — đường cũ của nó vẫn là đường cũ.
pub fn structured_fields(acc: &ClaudeAccountCfg) -> Vec<&'static str> {
    let mut v = Vec::new();
    if some(&acc.base_url).is_some() {
        v.push("base_url");
    }
    if some(&acc.env_file).is_some() {
        v.push("env_file");
    }
    if some(&acc.api_key_env).is_some() {
        v.push("api_key_env");
    }
    if some(&acc.api_key_as).is_some() {
        v.push("api_key_as");
    }
    if some(&acc.model).is_some() {
        v.push("model");
    }
    if some(&acc.effort).is_some() {
        v.push("effort");
    }
    if !acc.extra_env.is_empty() {
        v.push("extra_env");
    }
    if !acc.extra_args.is_empty() {
        v.push("extra_args");
    }
    if some(&acc.cli).is_some() {
        v.push("cli");
    }
    if acc.opencode_config.is_some() {
        v.push("opencode_config");
    }
    v
}

fn some(o: &Option<String>) -> Option<&str> {
    o.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// Tên thư mục phân vùng của một tài khoản opencode — chỉ chữ/số/`-`/`_`.
///
/// Tên tài khoản là thứ duy nhất phân biệt hai phân vùng, nên nó phải thành
/// một tên thư mục an toàn. Tên có dấu cách (hàng \"mặc định\") hoặc ký tự lạ
/// được thay bằng `_`, không bao giờ xoá — hai tên khác nhau không được gặp
/// nhau ở cùng một thư mục.
pub fn opencode_slug(acc: &ClaudeAccountCfg) -> String {
    let mut s: String = acc
        .name
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if s.is_empty() {
        s.push_str("default");
    }
    s
}

/// Phân vùng riêng của một tài khoản opencode: `~/.opencode-<tên>/`.
///
/// Bốn thư mục con là bốn biến XDG mà opencode đọc: `config`, `data`, `cache`,
/// `state`. Tách hẳn khỏi `~/.local/share/opencode` để hai tài khoản không
/// nhìn thấy phiên hay khoá của nhau.
pub fn opencode_profile_dir(acc: &ClaudeAccountCfg) -> PathBuf {
    match home_dir() {
        Some(h) => h.join(format!(".opencode-{}", opencode_slug(acc))),
        None => PathBuf::from(format!(".opencode-{}", opencode_slug(acc))),
    }
}

/// Bộ chặn quyền cho opencode, suy từ CHÍNH `DENIED_TOOLS` của huba.
///
/// Một nguồn duy nhất: danh sách Claude ở `sessions.rs` là thứ `/new` vẫn dùng;
/// nếu nó đổi mà bản opencode không đổi thì cửa sổ mở bằng hai CLI sẽ có hai
/// bức tường khác nhau. Phép biến đổi:
/// * `Bash(git push:*)` → shell pattern `git push *` (opencode V1: `permission.bash`).
/// * `Bash(*deploy*)`   → shell pattern `*deploy*` (đã là pattern).
/// * `WebFetch`/`WebSearch` → `webfetch`/`websearch: deny`.
fn opencode_permission_wall() -> Value {
    let mut bash = serde_json::Map::new();
    bash.insert("*".to_string(), json!("allow"));
    let mut extra = serde_json::Map::new();
    for tool in crate::sessions::denied_tools() {
        if let Some(body) = tool.strip_prefix("Bash(").and_then(|s| s.strip_suffix(")")) {
            let cmd = body.strip_suffix(":*").unwrap_or(body);
            let pattern = if cmd.contains('*') {
                cmd.to_string()
            } else {
                format!("{cmd} *")
            };
            bash.insert(pattern, json!("deny"));
        }
    }
    for tool in crate::sessions::denied_tools() {
        match tool.as_str() {
            "WebFetch" => {
                extra.insert("webfetch".to_string(), json!("deny"));
            }
            "WebSearch" => {
                extra.insert("websearch".to_string(), json!("deny"));
            }
            _ => {}
        }
    }
    let mut permission = serde_json::Map::new();
    permission.insert("bash".to_string(), Value::Object(bash));
    for (k, v) in extra {
        permission.insert(k, v);
    }
    Value::Object(permission)
}

/// Object JSON cuối cùng đi vào `OPENCODE_CONFIG_CONTENT`.
///
/// Thứ tự áp: cấu hình riêng của tài khoản trước, bộ chặn của huba SAU. Nhờ
/// vậy `opencode_config` thêm được provider/baseURL/agent, nhưng không mở lại
/// được `git push`, `sudo`, `rm`, WebFetch… — bức tường không phụ thuộc vào
/// một tệp người dùng sửa tay.
pub fn opencode_security_plugin() -> String {
    std::env::var("HUBA_OPENCODE_PLUGIN")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            home_dir()
                .map(|h| {
                    h.join("projects")
                        .join("scripts")
                        .join("cong-bao-mat-plugin")
                        .to_string_lossy()
                        .to_string()
                })
                .unwrap_or_default()
        })
}

/// Mở rộng `~/…` trong MỌI chuỗi của `opencode_config`, đệ quy.
///
/// 🔴 VÌ SAO: `huba.config.json` được COMMIT và kho origin là CÔNG KHAI, nên
/// đường dẫn tuyệt đối `/Users/<tên>/…` là dấu vết cá nhân (rào riêng tư của
/// repo chặn đúng — 12/11). Mà opencode KHÔNG tự mở `~` trong `plugin` / `mcp` /
/// `environment`, nên phải mở Ở ĐÂY, một chỗ: cấu hình giữ `~/…` (đọc được,
/// không lộ tên máy) còn tiến trình con nhận đường dẫn tuyệt đối.
fn expand_home_in_json(v: &mut Value) {
    match v {
        Value::String(s) if s.starts_with("~/") || s.starts_with("~\\") => {
            *s = crate::config::expand_home(std::path::Path::new(s))
                .to_string_lossy()
                .into_owned();
        }
        Value::Array(a) => a.iter_mut().for_each(expand_home_in_json),
        Value::Object(o) => o.values_mut().for_each(expand_home_in_json),
        _ => {}
    }
}

pub fn opencode_config_content(acc: &ClaudeAccountCfg) -> Value {
    let mut root = match &acc.opencode_config {
        Some(Value::Object(o)) => Value::Object(o.clone()),
        Some(_) => {
            crate::logging::warn(
                "account_opencode_config_ignored",
                json!({ "account": acc.name, "why": "opencode_config không phải object JSON" }),
            );
            json!({})
        }
        None => json!({}),
    };
    expand_home_in_json(&mut root);
    // Agent chỉ-đọc cho /ask + /handover. Đặt trước bộ chặn để tài khoản vẫn
    // có thể thêm agent riêng, nhưng agent này thì luôn có.
    let obj = root.as_object_mut().expect("root luôn là object");
    let agents = obj.entry("agent").or_insert_with(|| json!({}));
    if let Some(agents) = agents.as_object_mut() {
        agents.insert(
            "huba-fork".to_string(),
            json!({
                "description": "Huba: fork chỉ-đọc để hỏi một phiên đang sống",
                "mode": "primary",
                "permission": {
                    "bash": "deny",
                    "edit": "deny",
                    "webfetch": "deny",
                    "websearch": "deny",
                    "task": "deny"
                }
            }),
        );
    }
    let wall = opencode_permission_wall();
    let permission = obj.entry("permission").or_insert_with(|| json!({}));
    let allow = permission
        .as_object_mut()
        .expect("permission luôn là object");
    if let Some(wall) = wall.as_object() {
        for (k, v) in wall {
            if k == "bash" {
                let bash = allow.entry("bash").or_insert_with(|| json!({}));
                if let Some(bash) = bash.as_object_mut() {
                    if let Some(w) = v.as_object() {
                        for (pat, eff) in w {
                            // Chỉ áp các DENY; pattern "*" của bộ chặn không
                            // được đè lên lựa chọn rộng hơn của tài khoản.
                            if eff == "deny" {
                                bash.insert(pat.clone(), eff.clone());
                            }
                        }
                    }
                }
            } else {
                allow.insert(k.clone(), v.clone());
            }
        }
    }
    // `external_directory`: mặc định KHÔNG hỏi lại khi đụng thư mục ngoài
    // project đang mở, cho đúng cây workspace + vùng tạm. Đây là quyền ĐỌC/ĐI
    // thư mục; quyền GHI vẫn do cổng bảo mật (`cong_bao_mat.py`) quyết.
    // Không ghi đè lựa chọn chặt hơn của chủ máy trong `opencode_config`.
    if let Some(h) = home_dir() {
        if let Some(obj) = root.as_object_mut() {
            let perm = obj.entry("permission").or_insert_with(|| json!({}));
            if let Some(p) = perm.as_object_mut() {
                if !p.contains_key("external_directory") {
                    let ws = h.join("projects");
                    let ws_s = ws.to_string_lossy().to_string();
                    let mut m = serde_json::Map::new();
                    m.insert(ws_s.clone(), json!("allow"));
                    m.insert(format!("{ws_s}/*"), json!("allow"));
                    m.insert("/tmp/*".to_string(), json!("allow"));
                    m.insert("/private/tmp/*".to_string(), json!("allow"));
                    m.insert("/private/var/folders/*".to_string(), json!("allow"));
                    p.insert("external_directory".to_string(), Value::Object(m));
                }
            }
        }
    }
    // Model là trường `model` của tài khoản — TUI `opencode` không có cờ
    // `--model`, nên đây là cửa DUY NHẤT để cả `/new` lẫn `opencode run` dùng
    // đúng model. Trường này thắng `opencode_config.model` (nguồn sự thật).
    if let Some(m) = some(&acc.model) {
        if let Some(obj) = root.as_object_mut() {
            obj.insert("model".to_string(), json!(m));
        }
    }
    // Plugin bảo mật là MẶC ĐỊNH của mọi tài khoản opencode — không phụ thuộc
    // việc chủ máy nhớ khai nó trong `opencode_config`. Một cổng, mọi acc.
    let plugin = opencode_security_plugin();
    if !plugin.is_empty() {
        if !Path::new(&plugin).is_dir() {
            crate::logging::warn(
                "opencode_security_plugin_missing",
                json!({ "account": acc.name, "path": plugin,
                        "effect": "opencode sẽ không gọi được cổng bảo mật — sửa đường dẫn" }),
            );
        }
        if let Some(obj) = root.as_object_mut() {
            let arr = obj.entry("plugin").or_insert_with(|| json!([]));
            if let Some(arr) = arr.as_array_mut() {
                if !arr.iter().any(|v| v.as_str() == Some(plugin.as_str())) {
                    arr.push(json!(plugin));
                }
            }
        }
    }
    root
}

/// Dòng shell chủ máy VIẾT TAY (`launch`), đã cắt khoảng trắng thừa.
pub fn legacy_line(acc: &ClaudeAccountCfg) -> Option<&str> {
    some(&acc.launch)
}

/// Dòng lệnh THẮNG cho một tài khoản: `launch` viết tay nếu có, không thì
/// [`launch_line`] dựng từ trường cấu trúc.
///
/// Khi cả hai cùng có mặt, trường cấu trúc đang bị che ⇒ ghi `warn` kèm TÊN
/// trường (không bao giờ kèm giá trị — luật 4). Một `Err` thành mặc định mà
/// không có dòng log là bug (luật 3), và ở đây cái "mặc định" ấy là một tài
/// khoản KHÁC chạy thay.
pub fn effective_launch(cfg: &Config, acc: Option<&ClaudeAccountCfg>) -> String {
    let Some(acc) = acc else {
        return shell_quote(&cfg.claude_cli);
    };
    if let Some(l) = legacy_line(acc) {
        warn_if_shadowed(acc);
        return l.to_string();
    }
    launch_line(cfg, Some(acc))
}

/// [`effective_launch`] cho Windows PowerShell — `launch` viết tay đứng sau `&`.
pub fn effective_launch_ps(cfg: &Config, acc: Option<&ClaudeAccountCfg>) -> String {
    use crate::win_procs::ps_quote;
    let Some(acc) = acc else {
        return format!("& {}", ps_quote(&cfg.claude_cli));
    };
    if let Some(l) = legacy_line(acc) {
        warn_if_shadowed(acc);
        return format!("& {l}");
    }
    launch_line_ps(cfg, Some(acc))
}

fn warn_if_shadowed(acc: &ClaudeAccountCfg) {
    let shadowed = structured_fields(acc);
    if !shadowed.is_empty() {
        crate::logging::warn(
            "account_launch_legacy_shadows_fields",
            json!({ "account": acc.name, "fields": shadowed }),
        );
    }
}

/// Dòng lệnh mở phiên (macOS/Linux) dựng từ trường cấu trúc.
///
/// `None` = tài khoản mặc định (ambient): chỉ tên CLI, không biến nào.
/// Hàng KHÔNG khai trường cấu trúc nào ⇒ đường CŨ, byte-identical với
/// `sessions::account_launch` trước 09/11: `CLAUDE_CONFIG_DIR='<abs>' '<cli>'`.
///
/// Hàng có trường cấu trúc ⇒
/// `env <nạp tệp> <base_url> <extra_env đã sắp> CLAUDE_CONFIG_DIR=<dir>
///  <cli> <extra_args> [--model <m>]`.
pub fn launch_line(cfg: &Config, acc: Option<&ClaudeAccountCfg>) -> String {
    let Some(acc) = acc else {
        return shell_quote(&cfg.claude_cli);
    };
    if acc.is_opencode() {
        return opencode_launch_line(cfg, acc);
    }
    let dir = some(&acc.config_dir);
    if structured_fields(acc).is_empty() {
        return match dir {
            Some(d) => format!(
                "CLAUDE_CONFIG_DIR={} {}",
                shell_quote(&expand_home(Path::new(d)).to_string_lossy()),
                shell_quote(&cfg.claude_cli)
            ),
            None => shell_quote(&cfg.claude_cli),
        };
    }

    let mut parts: Vec<String> = Vec::new();
    let base_url = some(&acc.base_url);
    let env_file = some(&acc.env_file);
    if env_file.is_some() || base_url.is_some() || !acc.extra_env.is_empty() {
        parts.push("env".to_string());
        if let Some(f) = env_file {
            parts.push(env_load(f, some(&acc.api_key_env), some(&acc.api_key_as)));
        }
        if let Some(b) = base_url {
            parts.push(assign("ANTHROPIC_BASE_URL", b));
        }
        for (k, v) in &acc.extra_env {
            parts.push(assign(k, v));
        }
    }
    if let Some(d) = dir {
        parts.push(format!("CLAUDE_CONFIG_DIR={}", path_token(d)));
    }
    parts.push(shell_quote(&cfg.claude_cli));
    for a in &acc.extra_args {
        parts.push(value_token(a));
    }
    if let Some(m) = some(&acc.model) {
        parts.push("--model".to_string());
        parts.push(value_token(m));
    }
    parts.join(" ")
}

/// Dòng lệnh của một tài khoản `opencode` — mỗi tài khoản MỘT phân vùng.
///
/// Không có tệp cấu hình tĩnh: mọi thứ thay đổi được (khoá, model, provider)
/// đi qua `env`/`--model`, còn cấu hình runtime đi qua `OPENCODE_CONFIG_CONTENT`.
/// Nhờ vậy sửa trên trang `huba accounts` là lần mở SAU dùng ngay, không có bản
/// sao thứ hai để lệch.
///
/// Bốn biến XDG là ranh giới phân vùng: `~/.opencode-<tên>/{config,data,cache,state}`.
/// Khoá đọc từ `env_file` (chỉ ĐƯỜNG DẪN nằm trong cấu hình), model từ ô
/// `model`, và bức tường quyền từ [`opencode_permission_wall`].
fn opencode_launch_line(cfg: &Config, acc: &ClaudeAccountCfg) -> String {
    let mut parts: Vec<String> = vec!["env".to_string()];
    if let Some(f) = some(&acc.env_file) {
        parts.push(env_load(f, some(&acc.api_key_env), some(&acc.api_key_as)));
    } else if some(&acc.api_key_env).is_some() {
        crate::logging::warn(
            "account_opencode_key_without_file",
            json!({ "account": acc.name,
                    "why": "khai api_key_env mà thiếu env_file — biến chỉ có nếu môi trường ngoài đã đặt" }),
        );
    }
    let base = format!("$HOME/.opencode-{}", opencode_slug(acc));
    for (var, sub) in [
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_STATE_HOME", "state"),
    ] {
        parts.push(assign_path(var, &format!("{base}/{sub}")));
    }
    parts.push("OPENCODE_DISABLE_AUTOUPDATE=1".to_string());
    let content = serde_json::to_string(&opencode_config_content(acc)).unwrap_or_else(|e| {
        crate::logging::error(
            "account_opencode_config_encode_failed",
            json!({ "account": acc.name, "err": e.to_string() }),
        );
        "{}".to_string()
    });
    parts.push(format!("OPENCODE_CONFIG_CONTENT={}", shell_quote(&content)));
    for (k, v) in &acc.extra_env {
        if looks_secret(k) {
            crate::logging::warn(
                "account_env_secret_skipped",
                json!({ "account": acc.name, "var": k,
                        "why": "giá trị bí mật không đi qua dòng lệnh — dùng env_file" }),
            );
            continue;
        }
        parts.push(assign(k, v));
    }
    parts.push(path_token(&cfg.opencode_cli));
    for a in &acc.extra_args {
        parts.push(value_token(a));
    }
    // KHÔNG đặt `--model` ở đây: TUI `opencode` không nhận cờ này (chỉ
    // `opencode run` nhận), và cờ ấy làm cửa sổ `/new` chết ngay dòng đầu.
    // Model đi qua `OPENCODE_CONFIG_CONTENT` bên trên — một nguồn, cả TUI lẫn run.
    parts.join(" ")
}

/// [`launch_line`] cho Windows PowerShell.
///
/// `env_file` CHƯA có đường nạp trên Windows (`& claude` chạy không qua một
/// shell nào đọc tệp) ⇒ bỏ qua kèm `warn` có lý do, không nửa vời im lặng.
pub fn launch_line_ps(cfg: &Config, acc: Option<&ClaudeAccountCfg>) -> String {
    use crate::win_procs::ps_quote;
    let Some(acc) = acc else {
        return format!("& {}", ps_quote(&cfg.claude_cli));
    };
    if acc.is_opencode() {
        return opencode_launch_line_ps(cfg, acc);
    }
    let dir = some(&acc.config_dir);
    if structured_fields(acc).is_empty() {
        return match dir {
            Some(d) => format!(
                "$env:CLAUDE_CONFIG_DIR = {}; & {}",
                ps_quote(&expand_home(Path::new(d)).to_string_lossy()),
                ps_quote(&cfg.claude_cli)
            ),
            None => format!("& {}", ps_quote(&cfg.claude_cli)),
        };
    }

    let mut pre: Vec<String> = Vec::new();
    if some(&acc.env_file).is_some() {
        crate::logging::warn(
            "account_env_file_unsupported",
            json!({
                "account": acc.name,
                "why": "Windows: env_file chưa có đường nạp — biến trong tệp KHÔNG được đặt",
            }),
        );
    }
    if let Some(b) = some(&acc.base_url) {
        pre.push(format!("$env:ANTHROPIC_BASE_URL = {}", ps_quote(b)));
    }
    for (k, v) in &acc.extra_env {
        pre.push(format!("$env:{k} = {}", ps_quote(v)));
    }
    if let Some(d) = dir {
        pre.push(format!(
            "$env:CLAUDE_CONFIG_DIR = {}",
            ps_quote(&expand_home(Path::new(d)).to_string_lossy())
        ));
    }
    let mut call = format!("& {}", ps_quote(&cfg.claude_cli));
    for a in &acc.extra_args {
        call.push(' ');
        call.push_str(&ps_quote(a));
    }
    if let Some(m) = some(&acc.model) {
        call.push_str(&format!(" --model {}", ps_quote(m)));
    }
    if pre.is_empty() {
        call
    } else {
        format!("{}; {}", pre.join("; "), call)
    }
}

fn opencode_launch_line_ps(cfg: &Config, acc: &ClaudeAccountCfg) -> String {
    use crate::win_procs::ps_quote;
    if some(&acc.env_file).is_some() {
        crate::logging::warn(
            "account_env_file_unsupported",
            json!({ "account": acc.name,
                    "why": "Windows: env_file chưa có đường nạp — biến trong tệp KHÔNG được đặt" }),
        );
    }
    let base = opencode_profile_dir(acc);
    let mut pre: Vec<String> = Vec::new();
    for (var, sub) in [
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_STATE_HOME", "state"),
    ] {
        pre.push(format!(
            "$env:{var} = {}",
            ps_quote(&base.join(sub).to_string_lossy())
        ));
    }
    pre.push("$env:OPENCODE_DISABLE_AUTOUPDATE = '1'".to_string());
    let content =
        serde_json::to_string(&opencode_config_content(acc)).unwrap_or_else(|_| "{}".into());
    pre.push(format!(
        "$env:OPENCODE_CONFIG_CONTENT = {}",
        ps_quote(&content)
    ));
    for (k, v) in &acc.extra_env {
        if looks_secret(k) {
            continue;
        }
        pre.push(format!("$env:{k} = {}", ps_quote(v)));
    }
    let mut call = format!("& {}", ps_quote(&cfg.opencode_cli));
    for a in &acc.extra_args {
        call.push(' ');
        call.push_str(&ps_quote(a));
    }
    // Model đi qua `OPENCODE_CONFIG_CONTENT` — TUI opencode không nhận `--model`.
    format!("{}; {}", pre.join("; "), call)
}

/// Biến môi trường cho một tiến trình CON của tài khoản này.
///
/// Khác với dòng lệnh: giá trị phải có THẬT ở đây (đó là môi trường), nên
/// tuyệt đối không có giá trị bí mật — tên kết thúc `_KEY`/`_TOKEN`/`_SECRET`/
/// `_PASSWORD` bị BỎ và ghi `warn`. Khoá thì `env_file` + `apiKeyHelper` lo.
pub fn env_for_child(acc: &ClaudeAccountCfg) -> Vec<(String, Option<String>)> {
    if acc.is_opencode() {
        return opencode_env_for_child(acc);
    }
    let dir = some(&acc.config_dir).map(|d| expand_home(Path::new(d)).display().to_string());
    let mut out = vec![("CLAUDE_CONFIG_DIR".to_string(), dir)];
    if let Some(b) = some(&acc.base_url) {
        out.push(("ANTHROPIC_BASE_URL".to_string(), Some(b.to_string())));
    }
    for (k, v) in &acc.extra_env {
        if looks_secret(k) {
            crate::logging::warn(
                "account_env_secret_skipped",
                json!({ "account": acc.name, "var": k, "why": "giá trị bí mật không đi qua env của tiến trình con" }),
            );
            continue;
        }
        out.push((k.clone(), Some(v.clone())));
    }
    out
}

fn opencode_env_for_child(acc: &ClaudeAccountCfg) -> Vec<(String, Option<String>)> {
    let base = opencode_profile_dir(acc);
    let mut out = Vec::new();
    for (var, sub) in [
        ("XDG_CONFIG_HOME", "config"),
        ("XDG_DATA_HOME", "data"),
        ("XDG_CACHE_HOME", "cache"),
        ("XDG_STATE_HOME", "state"),
    ] {
        out.push((
            var.to_string(),
            Some(base.join(sub).to_string_lossy().to_string()),
        ));
    }
    out.push(("OPENCODE_DISABLE_AUTOUPDATE".to_string(), Some("1".into())));
    out.push((
        "OPENCODE_CONFIG_CONTENT".to_string(),
        Some(
            serde_json::to_string(&opencode_config_content(acc))
                .unwrap_or_else(|_| "{}".to_string()),
        ),
    ));
    for (k, v) in &acc.extra_env {
        if looks_secret(k) {
            crate::logging::warn(
                "account_env_secret_skipped",
                json!({ "account": acc.name, "var": k,
                        "why": "giá trị bí mật không đi qua env của tiến trình con" }),
            );
            continue;
        }
        out.push((k.clone(), Some(v.clone())));
    }
    out
}

pub fn looks_secret(key: &str) -> bool {
    let k = key.to_ascii_uppercase();
    ["_KEY", "_TOKEN", "_SECRET", "_PASSWORD"]
        .iter()
        .any(|s| k.ends_with(s))
}

/// Một khoá cần ghi vào `settings.json`, giữ ĐƯỜNG DẪN dạng mảng thay vì một
/// chuỗi có dấu chấm: tên model thật (`deepseek-v4.1-flash`) CÓ dấu chấm, nên
/// `"modelSettings.deepseek-v4.1-flash.effortLevel"` tách bằng `.` là hỏng.
#[derive(Debug, Clone, PartialEq)]
pub struct Setting {
    pub path: Vec<String>,
    pub value: Value,
}

impl Setting {
    pub fn new(path: &[&str], value: Value) -> Self {
        Setting {
            path: path.iter().map(|s| (*s).to_string()).collect(),
            value,
        }
    }
    /// Dạng hiển thị/so sánh — chỉ dùng để ĐỌC, không dùng để tách lại.
    pub fn dotted(&self) -> String {
        self.path.join(".")
    }
}

/// Bản vá `settings.json` mong muốn cho một tài khoản.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SettingsPatch {
    pub set: Vec<Setting>,
    /// Trường tài khoản khai mà `settings.json` KHÔNG mang được (kèm lý do) —
    /// để trang nói thẳng thay vì im lặng bỏ qua.
    pub unsupported: Vec<String>,
}

/// Dựng bản vá `settings.json` từ trường cấu trúc. `launch` legacy không tham
/// gia: dòng viết tay đã tự mang mọi thứ nó cần, chép vào settings là đoán.
pub fn settings_patch(acc: &ClaudeAccountCfg) -> SettingsPatch {
    if acc.is_opencode() {
        return SettingsPatch {
            set: Vec::new(),
            unsupported: vec![
                "opencode: cấu hình runtime đi qua OPENCODE_CONFIG_CONTENT, không dùng settings.json"
                    .to_string(),
            ],
        };
    }
    let mut p = SettingsPatch::default();
    if let Some(b) = some(&acc.base_url) {
        p.set
            .push(Setting::new(&["env", "ANTHROPIC_BASE_URL"], json!(b)));
    }
    for (k, v) in &acc.extra_env {
        if looks_secret(k) {
            p.unsupported.push(format!(
                "extra_env.{k} — giá trị bí mật không được ghi vào settings.json (luật 4)"
            ));
            continue;
        }
        p.set.push(Setting::new(&["env", k.as_str()], json!(v)));
    }
    match (some(&acc.api_key_env), some(&acc.api_key_as)) {
        (Some(k), None) => p.set.push(Setting::new(
            &["apiKeyHelper"],
            json!(format!("printenv {k}")),
        )),
        (Some(k), Some(as_)) => p.unsupported.push(format!(
            "api_key_as — settings.json không đổi tên biến ({k} → {as_}); cần `env_file` + dòng lệnh"
        )),
        (None, Some(_)) => p
            .unsupported
            .push("api_key_as — khai mà thiếu `api_key_env`".to_string()),
        (None, None) => {}
    }
    if let Some(m) = some(&acc.model) {
        p.set.push(Setting::new(&["model"], json!(m)));
        if let Some(e) = some(&acc.effort) {
            p.set
                .push(Setting::new(&["modelSettings", m, "effortLevel"], json!(e)));
        }
    } else if some(&acc.effort).is_some() {
        p.unsupported
            .push("effort — không có `model` thì không có khoá `modelSettings`".to_string());
    }
    if some(&acc.env_file).is_some() {
        p.unsupported.push(
            "env_file — tệp env chỉ đi qua dòng lệnh, không phải `settings.json`".to_string(),
        );
    }
    if !acc.extra_args.is_empty() {
        p.unsupported.push(
            "extra_args — đối số CLI chỉ đi qua dòng lệnh, không phải `settings.json`".to_string(),
        );
    }
    p
}

/// Khoá mà `settings.json` HIỆN CÓ đang khác bản vá, ở những chỗ thuộc về chủ
/// máy: `env.*` và `apiKeyHelper`. `model`/`effortLevel` khác là CẬP NHẬT bình
/// thường, không phải xung đột.
///
/// Trả về ĐƯỜNG DẪN, không bao giờ trả giá trị hiện có — giá trị ấy có thể là
/// khoá của chủ máy (luật 4).
pub fn settings_conflicts(current: &Value, patch: &SettingsPatch) -> Vec<String> {
    let mut out = Vec::new();
    for s in &patch.set {
        let owner = s.path.first().map(String::as_str) == Some("env")
            || (s.path.len() == 1 && s.path[0] == "apiKeyHelper");
        if !owner {
            continue;
        }
        if let Some(have) = json_at(current, &s.path) {
            if have != &s.value {
                out.push(s.dotted());
            }
        }
    }
    out
}

/// Đi theo ĐƯỜNG DẪN mảng trong một `Value` lồng nhau.
pub fn json_at<'a>(v: &'a Value, path: &[String]) -> Option<&'a Value> {
    let mut cur = v;
    for seg in path {
        cur = cur.get(seg)?;
    }
    Some(cur)
}

/// Kiểm TÊN tài khoản — chỉ gọi lúc GHI (thêm/sửa), không gọi lúc nạp cấu
/// hình: hàng mặc định tên `"mặc định"` có dấu cách, kiểm ở `validate()` sẽ
/// chặn khởi động của một máy đang chạy tốt.
pub fn validate_row(name: &str) -> Result<(), String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("tên tài khoản rỗng".into());
    }
    if n.len() > 40 {
        return Err(format!("tên `{n}` dài quá 40 ký tự"));
    }
    if !n
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(format!(
            "tên `{n}` chỉ nhận chữ, số, `_` và `-` (tên này thành `~/.claude-{n}`)"
        ));
    }
    Ok(())
}

// ── Cửa GHI (GĐ 3) ─────────────────────────────────────────────────────────

/// Kết quả một lượt [`upsert_account`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountWrite {
    Added,
    Updated,
}

/// Thêm hoặc sửa MỘT hàng tài khoản — đường ghi duy nhất của trang quản lý.
///
/// ⛔ **Không có hàm xoá**: chủ máy chốt 09/11 — *"xoá = chỉ khoá"*. Hàng đã có
/// thì SỬA tại chỗ; [`crate::config::edit_accounts`] là chỗ duy nhất có thể
/// thu ngắn danh sách, nên cả hai đường ghi đều không đi qua hình dạng ấy.
///
/// 🔒 **`locked` giữ nguyên khi SỬA.** Khoá là quyết định của chủ máy, không
/// phải một ô trong biểu mẫu: biểu mẫu không gửi khoá thì không được mở khoá
/// hộ. Mở/khoá có đường riêng ([`crate::pipeline::set_account_locked`]).
///
/// 🔴 Luật 4: hàm này chỉ nhận ĐƯỜNG DẪN và TÊN biến. Giá trị bí mật nằm
/// trong `env_file` (chmod 600), không bao giờ vào `huba.config.json`.
pub fn upsert_account(cfg: &Config, row: ClaudeAccountCfg) -> Result<AccountWrite, String> {
    validate_row(&row.name)?;
    if let Some(cli) = some(&row.cli) {
        if !cli.eq_ignore_ascii_case("claude") && !cli.eq_ignore_ascii_case("opencode") {
            return Err(format!("CLI `{cli}` không hợp lệ (chỉ claude/opencode)"));
        }
    }
    if let Some(v) = &row.opencode_config {
        if !v.is_object() {
            return Err("opencode_config phải là một object JSON".into());
        }
    }
    if row.is_opencode() {
        if let Some(m) = some(&row.model) {
            if !m.contains('/') {
                return Err(format!(
                    "model `{m}` phải có dạng `provider/model` cho opencode"
                ));
            }
        }
    }
    let mut moi = row;
    moi.name = moi.name.trim().to_string();
    let ten = moi.name.clone();
    let mut ket_qua = AccountWrite::Updated;
    crate::config::edit_accounts(cfg, |rows| {
        match rows.iter_mut().find(|r| r.name == ten) {
            Some(cu) => {
                moi.locked = cu.locked;
                *cu = moi.clone();
                ket_qua = AccountWrite::Updated;
            }
            None => {
                ket_qua = AccountWrite::Added;
                rows.push(moi.clone());
            }
        }
        Ok(())
    })
    .map_err(|e| format!("{e:#}"))?;
    log_write(
        "page",
        match ket_qua {
            AccountWrite::Added => "account_added",
            AccountWrite::Updated => "account_updated",
        },
        &ten,
        json!({ "fields": structured_fields(&moi) }),
    );
    Ok(ket_qua)
}

/// Ghi một dòng log cho MỖI lượt ghi tài khoản (luật 3 — không lượt ghi nào im).
/// Hình dạng cố định `{account, via, …}`; phần `extra` do nơi gọi đặt và
/// **không bao giờ chứa giá trị bí mật** (chỉ TÊN trường, TÊN biến).
pub fn log_write(via: &str, event: &str, name: &str, extra: Value) {
    let mut o = match extra {
        Value::Object(m) => m,
        _ => serde_json::Map::new(),
    };
    o.insert("account".into(), json!(name));
    o.insert("via".into(), json!(via));
    crate::logging::info(event, Value::Object(o));
}

// ── Khoá/mở · dựng thư mục · đồng bộ settings (GĐ 4) ───────────────────────

/// Đặt/bỏ khoá MỘT tài khoản — cửa DUY NHẤT, dùng chung cho buồng chat
/// (`pipeline::set_account_locked`, `via = "chat"`) và trang quản lý
/// (`via = "page"`). Gom về đây để log nói đúng CỬA nào đã bấm, thay vì mọi
/// lượt đều khai "chat".
///
/// ⛔ Vẫn KHÔNG có "xoá": khoá là thao tác mạnh nhất trên một hàng (Quang chốt
/// 09/11 — *"không xoá, chỉ khoá"*).
pub fn set_locked(cfg: &Config, name: &str, locked: bool, via: &str) -> anyhow::Result<String> {
    let mut truoc = false;
    crate::config::edit_accounts(cfg, |rows| {
        let biet: Vec<String> = rows.iter().map(|r| r.name.clone()).collect();
        let row = rows.iter_mut().find(|r| r.name == name).ok_or_else(|| {
            anyhow::anyhow!(
                "không có tài khoản '{}'. Máy này khai: {}",
                crate::exec::truncate(name, 24),
                biet.join(" · ")
            )
        })?;
        truoc = row.locked;
        row.locked = locked;
        Ok(())
    })?;
    log_write(
        via,
        "account_lock_set",
        name,
        json!({ "locked": locked, "truoc": truoc }),
    );
    let viec = match (truoc, locked) {
        (true, true) => "vẫn đang KHOÁ (không đổi)",
        (false, false) => "vẫn đang mở (không đổi)",
        (false, true) => {
            "đã KHOÁ — huba không tự chọn, không tự mở phiên, không tự dò /usage trên nó (gõ /accounts <tên> vẫn đo)"
        }
        (true, false) => "đã MỞ khoá — huba dùng lại được",
    };
    let dau = if locked { "🔒" } else { "🔓" };
    Ok(format!(
        "{dau} {name} {viec}\n(daemon nạp lại theo mtime, có hiệu lực từ vòng kế)"
    ))
}

/// Dựng thư mục cấu hình + liên kết `projects` cho một tài khoản MỚI — đúng ba
/// trạng thái của `scripts/them-tai-khoan.sh:55-71`.
///
/// 🔴 Trạng thái GIỮA không được tự sửa: `<dir>/projects` là THƯ MỤC THẬT nghĩa
/// là trong đó có thể đã có nhật ký phiên thật. Gộp hai kho nhật ký là việc phải
/// nhìn tận mắt — hàm này DỪNG và nói số mục, không tự xoá.
pub fn ensure_dirs(acc: &ClaudeAccountCfg) -> Result<Vec<String>, String> {
    if acc.is_opencode() {
        let base = opencode_profile_dir(acc);
        let mut notes = Vec::new();
        for sub in ["config", "data", "cache", "state"] {
            let p = base.join(sub);
            std::fs::create_dir_all(&p)
                .map_err(|e| format!("không tạo được thư mục {}: {e}", p.display()))?;
            notes.push(format!("phân vùng opencode: {}", p.display()));
        }
        return Ok(notes);
    }
    if acc.config_dir.as_deref().unwrap_or("").trim().is_empty() {
        return Err(
            "không có `config_dir` — trang không dựng thư mục cho tài khoản MẶC ĐỊNH (nó là `~/.claude`)"
                .into(),
        );
    }
    let dir = crate::api_key::thu_muc(acc);
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("không tạo được thư mục {}: {e}", dir.display()))?;
    let mut notes = vec![format!("thư mục cấu hình: {}", dir.display())];

    // Liên kết mềm là chuyện của Unix — trên Windows hàm này chỉ NÓI, không tạo,
    // nên `goc` (và `$HOME`) không được đọc ở đó: biến thừa là warning, mà
    // `-D warnings` của cổng Windows biến nó thành lỗi build.
    #[cfg(unix)]
    let goc = match home_dir() {
        Some(home) => home.join(".claude").join("projects"),
        None => return Err("không đọc được $HOME để dựng liên kết `projects`".into()),
    };
    let projects = dir.join("projects");
    match std::fs::symlink_metadata(&projects) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            #[cfg(unix)]
            {
                std::os::unix::fs::symlink(&goc, &projects)
                    .map_err(|e| format!("không tạo được liên kết {}: {e}", projects.display()))?;
                notes.push(format!("projects → {} (vừa tạo)", goc.display()));
            }
            #[cfg(not(unix))]
            notes.push(
                "projects: nền này chưa hỗ trợ tạo liên kết — phải tạo tay TRƯỚC lượt `claude` đầu tiên"
                    .to_string(),
            );
        }
        Err(e) => return Err(format!("không đọc được {}: {e}", projects.display())),
        Ok(m) if m.file_type().is_symlink() => {
            #[cfg(unix)]
            {
                let dang_tro = std::fs::read_link(&projects).ok();
                if dang_tro.as_deref() == Some(goc.as_path()) {
                    notes.push(format!("projects → {} (đã đúng)", goc.display()));
                } else {
                    std::fs::remove_file(&projects).map_err(|e| {
                        format!("không gỡ được liên kết cũ {}: {e}", projects.display())
                    })?;
                    std::os::unix::fs::symlink(&goc, &projects)
                        .map_err(|e| format!("không trỏ lại được {}: {e}", projects.display()))?;
                    notes.push(format!("projects → {} (đã trỏ lại)", goc.display()));
                }
            }
            #[cfg(not(unix))]
            notes.push(format!(
                "projects: đang là liên kết (→ {}), nền này không kiểm lại được",
                std::fs::read_link(&projects)
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "?".into())
            ));
        }
        Ok(_) => {
            let n = std::fs::read_dir(&projects).map(|d| d.count()).unwrap_or(0);
            return Err(format!(
                "⛔ {} đang là THƯ MỤC THẬT, không phải liên kết ({n} mục). Trong đó có thể là nhật ký \
                 phiên thật — gộp tay rồi làm lại; huba KHÔNG tự xoá nhật ký của ai.",
                projects.display()
            ));
        }
    }
    Ok(notes)
}

/// Kết quả một lượt [`apply_settings`] — để trang nói được nó vừa làm gì.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsApplied {
    pub path: String,
    /// Đường dẫn các khoá ĐÃ ghi (dạng `a.b.c`), không phải giá trị.
    pub set: Vec<String>,
    /// Trường chỉ đi qua DÒNG LỆNH (không mang được vào settings.json).
    pub unsupported: Vec<String>,
    pub backup: Option<String>,
}

/// Đồng bộ `settings.json` của tài khoản với TRƯỜNG CẤU TRÚC (nguồn sự thật).
///
/// Bốn tính chất, mỗi cái là một lần đoán sai nếu bỏ:
/// * **Merge-only**: chỉ ghi những khoá bản vá khai; khoá khác trong
///   `settings.json` của chủ máy giữ nguyên.
/// * **Từ chối khi xung đột**: `env.ANTHROPIC_BASE_URL` / `apiKeyHelper` đang
///   mang một giá trị KHÁC giá trị cấu hình khai ⇒ KHÔNG ghi đè, trả lỗi kèm
///   ĐƯỜNG DẪN (không bao giờ giá trị — luật 4). Chủ máy sửa tay rồi đồng bộ lại.
/// * **Ghi rồi mới đổi tên** (`.tmp` → tệp) và `settings.json.bak` trước khi
///   đổi — một lần ghi hỏng giữa đường không để lại tệp cấu hình nửa vời.
/// * **Tệp hỏng thì DỪNG**, không đè: `settings.json` không phải JSON là một
///   tệp huba không hiểu, và ghi đè nó là xoá cấu hình của chủ máy.
pub fn apply_settings(acc: &ClaudeAccountCfg) -> Result<SettingsApplied, String> {
    let dir = crate::api_key::thu_muc(acc);
    let sp = dir.join("settings.json");
    let sps = sp.display().to_string();
    let patch = settings_patch(acc);

    // Không có khoá nào để ghi ⇒ KHÔNG tạo/đụng tệp (khớp `check_settings`).
    if patch.set.is_empty() {
        return Ok(SettingsApplied {
            path: sps,
            set: Vec::new(),
            unsupported: patch.unsupported,
            backup: None,
        });
    }

    let mut current: Value = match std::fs::read_to_string(&sp) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => json!({}),
        Err(e) => return Err(format!("không đọc được {sps}: {e}")),
        Ok(text) => serde_json::from_str(&text).map_err(|e| {
            format!("{sps} không phải JSON hợp lệ — KHÔNG ghi đè, sửa tay trước: {e}")
        })?,
    };
    if !current.is_object() {
        return Err(format!("{sps} không phải một object JSON — KHÔNG ghi đè"));
    }

    let conflicts = settings_conflicts(&current, &patch);
    if !conflicts.is_empty() {
        return Err(format!(
            "từ chối ghi đè {} — settings.json đang mang giá trị KHÁC cấu hình khai. Sửa tay khoá ấy \
             (hoặc bỏ nó) rồi bấm đồng bộ lại.",
            conflicts.join(", ")
        ));
    }

    let mut backup = None;
    if sp.is_file() {
        let bak = sp.with_extension("json.bak");
        std::fs::copy(&sp, &bak)
            .map_err(|e| format!("không sao lưu được {}: {e}", bak.display()))?;
        backup = Some(bak.display().to_string());
    }
    for s in &patch.set {
        set_at(&mut current, &s.path, s.value.clone());
    }
    let text = serde_json::to_string_pretty(&current)
        .map_err(|e| format!("không dựng lại được JSON: {e}"))?
        + "\n";
    let tmp = sp.with_extension("json.tmp");
    std::fs::write(&tmp, text).map_err(|e| format!("không ghi được {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &sp).map_err(|e| format!("không đổi tên vào {}: {e}", sp.display()))?;

    let set: Vec<String> = patch.set.iter().map(|s| s.dotted()).collect();
    log_write(
        "page",
        "account_settings_applied",
        &acc.name,
        json!({
            "path": sps,
            "set": set,
            "unsupported": patch.unsupported,
            "backup": backup,
        }),
    );
    Ok(SettingsApplied {
        path: sps,
        set,
        unsupported: patch.unsupported,
        backup,
    })
}

/// Ghi `val` vào ĐƯỜNG DẪN mảng trong một `Value`, tự dựng object trung gian.
/// Đệ quy để không vướng vòng mượn `&mut` khi đi sâu.
fn set_at(v: &mut Value, path: &[String], val: Value) {
    if path.is_empty() {
        *v = val;
        return;
    }
    if !v.is_object() {
        *v = json!({});
    }
    let map = v.as_object_mut().expect("vừa bảo đảm là object");
    let child = map.entry(path[0].clone()).or_insert_with(|| json!({}));
    set_at(child, &path[1..], val);
}

// ── chi tiết dựng chuỗi ────────────────────────────────────────────────────

/// MỘT dòng `env` nạp tệp: `$(cat …|xargs)`, có `sed` đổi tên khi `api_key_as`.
///
/// ⚠ `xargs` tách theo khoảng trắng và bỏ nháy — đúng cơ chế acc9 vẫn chạy từ
/// trước, giữ nguyên để dòng dựng khớp dòng đang chạy thật. Một giá trị có dấu
/// cách sẽ vỡ; đó là lý do `settings_patch` KHÔNG đi đường này.
fn env_load(file: &str, key_env: Option<&str>, key_as: Option<&str>) -> String {
    let path = path_token(file);
    match (key_env, key_as) {
        (Some(k), Some(as_)) => format!("$(sed \"s/^{k}=/{as_}=/\" {path} | xargs)"),
        _ => format!("$(cat {path} | xargs)"),
    }
}

fn assign(key: &str, value: &str) -> String {
    format!("{key}={}", value_token(value))
}

/// Gán một biến là ĐƯỜNG DẪN — `$HOME/...` phải còn `$` để shell giãn.
fn assign_path(key: &str, value: &str) -> String {
    format!("{key}={}", path_token(value))
}

/// Từ an toàn để để TRẦN trên dòng lệnh. Không có `$`, nháy, khoảng trắng,
/// `;`, `|`, `&` ⇒ shell không có gì để giãn.
fn is_safe_token(s: &str) -> bool {
    !s.is_empty()
        && s.chars().all(|c| {
            c.is_ascii_alphanumeric()
                || matches!(c, '.' | '_' | '-' | '/' | ':' | '@' | '+' | ',' | '=')
        })
}

fn value_token(v: &str) -> String {
    if is_safe_token(v) {
        v.to_string()
    } else {
        shell_quote(v)
    }
}

/// Đường dẫn in ra dạng `$HOME/...` khi nó nằm dưới nhà — đúng thứ chủ máy gõ
/// ở terminal, và đúng thứ `~/.zshrc` của chủ máy dùng.
///
/// 🔴 KHÔNG bọc `$HOME/...` bằng `shell_quote`: nháy ĐƠN làm shell thôi giãn
/// `$HOME`, và `CLAUDE_CONFIG_DIR='$HOME/.claude-acc9'` là một thư mục không
/// tồn tại. Phần còn lại không an toàn thì rơi về đường tuyệt đối đã bọc.
fn path_token(raw: &str) -> String {
    let raw = raw.trim();
    for home in ["$HOME/", "~/"] {
        if let Some(rest) = raw.strip_prefix(home) {
            if is_safe_token(rest) {
                return format!("$HOME/{rest}");
            }
        }
    }
    let abs = expand_home(Path::new(raw));
    let s = abs.to_string_lossy().into_owned();
    if let Some(home) = home_dir() {
        let hs = home.to_string_lossy().into_owned();
        if let Some(rest) = s.strip_prefix(&hs).and_then(|r| r.strip_prefix('/')) {
            if is_safe_token(rest) {
                return format!("$HOME/{rest}");
            }
        }
    }
    shell_quote(&s)
}

/// `true` khi hàng có ít nhất một trường cấu trúc — dùng cho trang quản lý.
pub fn has_structured(acc: &ClaudeAccountCfg) -> bool {
    !structured_fields(acc).is_empty()
}

/// Bảng tên → hàng, cho chỗ chỉ cần tra cứu.
pub fn by_name<'a>(accs: &'a [ClaudeAccountCfg], name: &str) -> Option<&'a ClaudeAccountCfg> {
    accs.iter().find(|a| a.name == name)
}

// ── GĐ 2 — CHẨN ĐOÁN: "tài khoản này thiếu gì" ─────────────────────────────
//
// Gốc: Quang 09/11 *"vụ acc9 không có đầu vào cố định dẫn đến không biết lỗi do
// đâu"*. Trước GĐ 2, một hàng hỏng chỉ lộ ra LÚC CHẠY (cửa sổ mở ra rồi đứng).
// Ở đây mỗi hàng trả lời được 8 câu, mỗi câu kèm đường dẫn để chủ máy tự đo lại.
//
// 🔴 Ba trạng thái, không hai (luật 13②): "không đo được" là trạng thái RIÊNG.
// Gộp nó vào "hỏng" là bịa một lỗi; gộp vào "tốt" là giấu một chỗ mù.

/// Trạng thái một mục chẩn đoán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckState {
    Ok,
    Fail,
    /// Phép đo hỏng / không đủ dữ liệu — KHÔNG phải "hỏng", KHÔNG phải "tốt".
    Unknown,
}

/// Một mục chẩn đoán. `text` viết cho NGƯỜI ĐỌC và không bao giờ chứa giá trị
/// bí mật (luật 4); `path` là chỗ tự đo lại.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    pub code: &'static str,
    pub state: CheckState,
    pub text: String,
    pub path: String,
}

impl Check {
    pub fn ok(&self) -> bool {
        self.state == CheckState::Ok
    }

    pub fn icon(&self) -> &'static str {
        match self.state {
            CheckState::Ok => "✓",
            CheckState::Fail => "✗",
            CheckState::Unknown => "⚠",
        }
    }

    /// Một dòng để in — đã gồm cả đường dẫn đo lại.
    pub fn line(&self) -> String {
        format!("{} {}", self.icon(), self.text)
    }
}

/// Dữ liệu THÔ của một tài khoản, do [`probe`] đọc từ đĩa. `checks` chỉ đọc
/// struct này nên chấm được bằng bài kiểm không chạm `$HOME`.
#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    /// Thư mục cấu hình THẬT (`config_dir`, hoặc `~/.claude` khi hàng mặc định).
    pub dir: String,
    pub dir_exists: bool,
    pub projects: ProjectsState,
    pub env_file: EnvFileState,
    pub settings: SettingsState,
    /// `None` = chưa đo sổ hạn mức (vd `doctor` không muốn đọc `$HOME`).
    pub quota: Option<Quota>,
    /// Nhãn [`crate::api_key::nhan`] — `None` = không có `apiKeyHelper`.
    pub api_key: Option<String>,
    /// `Some(0)` là một con số THẬT; `None` = chưa đo.
    pub live_sessions: Option<usize>,
    /// Lượt này KHÔNG liệt kê được phiên của tài khoản — kèm lý do (luật 11b).
    pub blind: Option<String>,
}

impl Default for Probe {
    fn default() -> Self {
        Probe {
            dir: String::new(),
            dir_exists: false,
            projects: ProjectsState::Missing,
            env_file: EnvFileState::NotDeclared,
            settings: SettingsState::Missing(String::new()),
            quota: None,
            api_key: None,
            live_sessions: None,
            blind: None,
        }
    }
}

/// Trạng thái `<config_dir>/projects` — chỗ acc8/acc9 từng làm hỏng nhật ký.
#[derive(Debug, Clone, PartialEq)]
pub enum ProjectsState {
    Missing,
    Link(PathBuf),
    /// Thư mục THẬT: nhật ký trong đó vô hình với huba, và gộp tay là việc phải
    /// nhìn tận mắt (`scripts/them-tai-khoan.sh` cũng dừng ở đây).
    RealDir,
    Unreadable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvFileState {
    NotDeclared,
    Missing(String),
    Unreadable {
        path: String,
        why: String,
    },
    /// `vars` là DANH SÁCH TÊN biến — không bao giờ giá trị.
    Found {
        path: String,
        mode: Option<u32>,
        vars: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum SettingsState {
    Missing(String),
    Bad { path: String, why: String },
    Found(Value),
}

#[cfg(unix)]
fn file_mode(m: &std::fs::Metadata) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    Some(m.permissions().mode() & 0o777)
}

#[cfg(not(unix))]
fn file_mode(_: &std::fs::Metadata) -> Option<u32> {
    None
}

/// Đọc đĩa cho MỘT tài khoản. Chỉ đọc — không ghi, không chạy tiến trình.
///
/// `quota` do chỗ gọi truyền vào: `account_detail_say` đã đọc sổ hạn mức rồi,
/// đọc lại ở đây sẽ log hai lần cùng một lỗi (luật 3 nhưng ồn).
pub fn probe(acc: &ClaudeAccountCfg, quota: Option<Quota>) -> Probe {
    use std::io::ErrorKind;

    let dir = if acc.is_opencode() {
        opencode_profile_dir(acc)
    } else {
        crate::api_key::thu_muc(acc)
    };
    let dir_s = dir.display().to_string();
    let dir_exists = dir.is_dir();

    let projects_path = dir.join("projects");
    let projects = match std::fs::symlink_metadata(&projects_path) {
        Err(e) if e.kind() == ErrorKind::NotFound => ProjectsState::Missing,
        Err(e) => ProjectsState::Unreadable(format!("{}: {e}", projects_path.display())),
        Ok(m) if m.file_type().is_symlink() => match std::fs::read_link(&projects_path) {
            Ok(t) => ProjectsState::Link(t),
            Err(e) => ProjectsState::Unreadable(format!("{}: {e}", projects_path.display())),
        },
        Ok(_) => ProjectsState::RealDir,
    };

    let env_file = match some(&acc.env_file) {
        None => EnvFileState::NotDeclared,
        Some(raw) => {
            let p = expand_home(Path::new(raw));
            let ps = p.display().to_string();
            match std::fs::metadata(&p) {
                Err(e) if e.kind() == ErrorKind::NotFound => EnvFileState::Missing(ps),
                Err(e) => EnvFileState::Unreadable {
                    path: ps,
                    why: e.to_string(),
                },
                Ok(m) => match std::fs::read_to_string(&p) {
                    Err(e) => EnvFileState::Unreadable {
                        path: ps,
                        why: e.to_string(),
                    },
                    Ok(text) => EnvFileState::Found {
                        path: ps,
                        mode: file_mode(&m),
                        vars: env_var_names(&text),
                    },
                },
            }
        }
    };

    let sp = dir.join("settings.json");
    let sps = sp.display().to_string();
    let settings = match std::fs::read_to_string(&sp) {
        Err(e) if e.kind() == ErrorKind::NotFound => SettingsState::Missing(sps),
        Err(e) => SettingsState::Bad {
            path: sps,
            why: e.to_string(),
        },
        Ok(text) => match serde_json::from_str::<Value>(&text) {
            Ok(v @ Value::Object(_)) => SettingsState::Found(v),
            Ok(_) => SettingsState::Bad {
                path: sps,
                why: "không phải một object JSON".into(),
            },
            Err(e) => SettingsState::Bad {
                path: sps,
                why: e.to_string(),
            },
        },
    };

    let (live_sessions, blind) = if acc.is_opencode() {
        match crate::opencode::count_sessions(acc) {
            Ok(n) => (Some(n), None),
            Err(e) => (None, Some(e)),
        }
    } else {
        (None, None)
    };
    Probe {
        dir: dir_s,
        dir_exists,
        projects,
        env_file,
        settings,
        quota,
        api_key: if acc.is_opencode() {
            None
        } else {
            crate::api_key::nhan(&dir)
        },
        live_sessions,
        blind,
    }
}

/// TÊN biến trong tệp env — `KEY=VALUE` ⇒ `KEY`. Bỏ dòng trống và `#`.
/// CỐ Ý không lấy giá trị: nó là bí mật và không có mục chẩn đoán nào cần.
fn env_var_names(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().to_string()))
        .filter(|k| !k.is_empty())
        .collect()
}

/// 8 mục chẩn đoán của MỘT tài khoản — hàm THUẦN trên [`Probe`].
///
/// Thứ tự cố định: thư mục → nhật ký → đăng nhập → API key → tệp env →
/// `settings.json` → `launch` → phiên sống.
pub fn checks(acc: &ClaudeAccountCfg, p: &Probe) -> Vec<Check> {
    if acc.is_opencode() {
        return checks_opencode(acc, p);
    }
    vec![
        check_dir(p),
        check_projects(p),
        check_login(p),
        check_api_key(acc, p),
        check_env_file(acc, p),
        check_settings(acc, p),
        check_launch(acc),
        check_live(p),
    ]
}

/// Chẩn đoán cho một hàng `opencode`: không có `settings.json`, không có
/// OAuth, không có sổ nhật ký Claude — nên 8 mục của Claude sẽ báo đỏ oan.
fn checks_opencode(acc: &ClaudeAccountCfg, p: &Probe) -> Vec<Check> {
    vec![
        check_profile(p),
        check_env_file(acc, p),
        check_opencode_key(acc, p),
        check_opencode_model(acc),
        check_opencode_config(acc),
        check_launch(acc),
        check_live(p),
    ]
}

fn check_profile(p: &Probe) -> Check {
    let state = if p.dir_exists {
        CheckState::Ok
    } else {
        // Không phải lỗi: lần mở đầu tiên tạo phân vùng. Nói đúng trạng thái.
        CheckState::Unknown
    };
    Check {
        code: "profile",
        state,
        text: if p.dir_exists {
            format!("phân vùng opencode: {}", p.dir)
        } else {
            format!("phân vùng CHƯA có (sẽ tạo ở lần mở đầu): {}", p.dir)
        },
        path: p.dir.clone(),
    }
}

fn check_opencode_key(acc: &ClaudeAccountCfg, p: &Probe) -> Check {
    let want = some(&acc.api_key_env);
    let (state, text) = match (&p.env_file, want) {
        (EnvFileState::NotDeclared, _) => (
            CheckState::Fail,
            "chưa khai `env_file` — opencode không có khoá".to_string(),
        ),
        (EnvFileState::Missing(f), _) => (CheckState::Fail, format!("tệp env không tồn tại: {f}")),
        (EnvFileState::Unreadable { path, why }, _) => {
            (CheckState::Fail, format!("không đọc được {path}: {why}"))
        }
        (EnvFileState::Found { path, vars, .. }, Some(k)) => {
            if vars.iter().any(|v| v == k) {
                (CheckState::Ok, format!("tệp env có biến `{k}`: {path}"))
            } else {
                (
                    CheckState::Fail,
                    format!("tệp env KHÔNG có biến `{k}`: {path}"),
                )
            }
        }
        (EnvFileState::Found { path, .. }, None) => (
            CheckState::Unknown,
            format!("có tệp env nhưng chưa khai `api_key_env`: {path}"),
        ),
    };
    Check {
        code: "opencode_key",
        state,
        text,
        path: acc
            .env_file
            .clone()
            .unwrap_or_else(|| "(chưa khai env_file)".into()),
    }
}

fn check_opencode_model(acc: &ClaudeAccountCfg) -> Check {
    let model = some(&acc.model);
    let (state, text) = match model {
        None => (
            CheckState::Fail,
            "chưa khai `model` — mở phiên sẽ dùng mặc định của opencode".to_string(),
        ),
        Some(m) if m.contains('/') => (CheckState::Ok, format!("model: {m}")),
        Some(m) => (
            CheckState::Fail,
            format!("model `{m}` thiếu `provider/` — opencode cần dạng provider/model"),
        ),
    };
    Check {
        code: "opencode_model",
        state,
        text,
        path: "(cấu hình tài khoản)".into(),
    }
}

fn check_opencode_config(acc: &ClaudeAccountCfg) -> Check {
    let (state, text) = match &acc.opencode_config {
        None => (
            CheckState::Ok,
            "cấu hình riêng: không khai (dùng mặc định)".to_string(),
        ),
        Some(Value::Object(_)) => (
            CheckState::Ok,
            "cấu hình riêng: object JSON hợp lệ".to_string(),
        ),
        Some(_) => (
            CheckState::Fail,
            "`opencode_config` phải là một object JSON".to_string(),
        ),
    };
    Check {
        code: "opencode_config",
        state,
        text,
        path: "(cấu hình tài khoản)".into(),
    }
}

/// Các mục KHÔNG đạt (hỏng hoặc không đo được), theo đúng thứ tự của [`checks`].
pub fn problems(checks: &[Check]) -> Vec<&Check> {
    checks.iter().filter(|c| !c.ok()).collect()
}

fn check_dir(p: &Probe) -> Check {
    let (state, text) = if p.dir_exists {
        (CheckState::Ok, format!("thư mục cấu hình có: {}", p.dir))
    } else {
        (
            CheckState::Fail,
            format!(
                "KHÔNG có {} — `claude` trong đó sẽ tự tạo và đăng nhập riêng",
                p.dir
            ),
        )
    };
    Check {
        code: "dir",
        state,
        text,
        path: p.dir.clone(),
    }
}

fn check_projects(p: &Probe) -> Check {
    if !p.dir_exists {
        return Check {
            code: "projects",
            state: CheckState::Unknown,
            text: format!("chưa có thư mục cấu hình — chưa kiểm {}", p.dir),
            path: p.dir.clone(),
        };
    }
    let here = format!("{}/projects", p.dir);
    let expected = home_dir().map(|h| h.join(".claude").join("projects"));
    let same = |a: &Path, b: &Path| {
        a == b
            || std::fs::canonicalize(a)
                .ok()
                .zip(std::fs::canonicalize(b).ok())
                .is_some_and(|(x, y)| x == y)
    };
    let (state, text) = match &p.projects {
        ProjectsState::Missing => (
            CheckState::Fail,
            format!(
                "chưa có {here} — nhật ký phiên nằm ngoài kho chung; tạo: ln -s ~/.claude/projects {here}"
            ),
        ),
        ProjectsState::Link(t) => match &expected {
            Some(e) if same(t, e) => (CheckState::Ok, format!("{here} → {}", t.display())),
            _ => (
                CheckState::Fail,
                format!(
                    "{here} → {} — KHÔNG phải ~/.claude/projects, nhật ký phiên vô hình với huba",
                    t.display()
                ),
            ),
        },
        ProjectsState::RealDir => (
            CheckState::Fail,
            format!(
                "{here} là THƯ MỤC THẬT, không phải liên kết — nhật ký trong đó vô hình với huba; \
                 gộp tay rồi đổi thành liên kết (scripts/them-tai-khoan.sh dừng đúng ở đây)"
            ),
        ),
        ProjectsState::Unreadable(why) => (
            CheckState::Unknown,
            format!("không đọc được {here}: {why}"),
        ),
    };
    Check {
        code: "projects",
        state,
        text,
        path: here,
    }
}

fn check_login(p: &Probe) -> Check {
    let path = format!("{}/.claude.json", p.dir);
    // Chạy `apiKeyHelper` thì sổ KHÔNG có `oauthAccount` là đúng — chấm ✗ ở đây
    // là một lời báo đỏ bịa. Chỉ in NHÃN, không bao giờ in khoá (luật 4).
    if let Some(label) = &p.api_key {
        return Check {
            code: "login",
            state: CheckState::Ok,
            text: format!("dùng {label} — không cần đăng nhập gói"),
            path: format!("{}/settings.json", p.dir),
        };
    }
    let (state, text) = match &p.quota {
        None => (
            CheckState::Unknown,
            "chưa đo sổ hạn mức — chưa biết đăng nhập được chưa".to_string(),
        ),
        // 🔴 Nguyên văn lý do của `quota::read`, không tự bịa 0%.
        Some(q) => match (&q.chua_dung_duoc, &q.why_unknown) {
            (Some(why), _) => (CheckState::Fail, why.clone()),
            (None, Some(why)) => (
                CheckState::Unknown,
                format!("không đọc được sổ hạn mức: {why}"),
            ),
            (None, None) => (CheckState::Ok, "sổ hạn mức đọc được".to_string()),
        },
    };
    Check {
        code: "login",
        state,
        text,
        path,
    }
}

fn check_api_key(acc: &ClaudeAccountCfg, p: &Probe) -> Check {
    let dir_settings = format!("{}/settings.json", p.dir);
    let (state, text) = match (some(&acc.api_key_env), some(&acc.api_key_as)) {
        (None, _) => match &p.api_key {
            // Đang chạy key mà cấu hình không khai: trang không có đường sửa.
            Some(label) => (
                CheckState::Fail,
                format!(
                    "đang chạy {label} nhưng cấu hình KHÔNG khai `api_key_env` — \
                     trang không biết key nào, sửa tay ở {dir_settings}"
                ),
            ),
            None => (
                CheckState::Ok,
                "dùng gói đăng nhập (không có `apiKeyHelper`)".to_string(),
            ),
        },
        (Some(k), Some(as_)) => (
            CheckState::Ok,
            format!("API key nạp từ `env_file`, đổi tên {k} → {as_} lúc chạy"),
        ),
        // Khai `api_key_env` mà settings không có helper: CLI chạy bằng gói,
        // còn key khai trong cấu hình không đi tới đâu cả.
        (Some(k), None) => match &p.api_key {
            Some(label) => (CheckState::Ok, format!("{label} · helper `printenv {k}`")),
            None => (
                CheckState::Fail,
                format!(
                    "khai `api_key_env = {k}` mà {dir_settings} KHÔNG có \
                     `apiKeyHelper: printenv {k}` — CLI chạy bằng gói, key không được nạp"
                ),
            ),
        },
    };
    Check {
        code: "api_key",
        state,
        text,
        path: dir_settings,
    }
}

fn check_env_file(acc: &ClaudeAccountCfg, p: &Probe) -> Check {
    let path = p.dir.clone();
    let (state, text) = match &p.env_file {
        EnvFileState::NotDeclared => (
            CheckState::Ok,
            "không khai `env_file` — dòng lệnh không nạp tệp nào".to_string(),
        ),
        EnvFileState::Missing(f) => (
            CheckState::Fail,
            format!("khai `env_file` mà KHÔNG có tệp {f} — dòng lệnh sẽ nạp rỗng"),
        ),
        EnvFileState::Unreadable { path: f, why } => {
            (CheckState::Fail, format!("không đọc được {f}: {why}"))
        }
        EnvFileState::Found {
            path: f,
            mode,
            vars,
        } => {
            let mut hong: Vec<String> = Vec::new();
            match mode {
                Some(m) if *m != 0o600 => hong.push(format!("quyền {m:03o} — phải 600")),
                Some(_) => {}
                None => {}
            }
            if let Some(k) = some(&acc.api_key_env) {
                if !vars.iter().any(|v| v == k) {
                    hong.push(format!("không có dòng `{k}=`"));
                }
            }
            let bien = some(&acc.api_key_env)
                .map(|k| format!(" · có {k}"))
                .unwrap_or_default();
            if hong.is_empty() {
                match mode {
                    Some(_) => (
                        CheckState::Ok,
                        format!("tệp 600 · {} biến{bien}", vars.len()),
                    ),
                    None => (
                        CheckState::Unknown,
                        format!(
                            "hệ này không đọc được quyền tệp (không kiểm 600) · {} biến{bien}",
                            vars.len()
                        ),
                    ),
                }
            } else {
                (CheckState::Fail, format!("{f}: {}", hong.join(" · ")))
            }
        }
    };
    Check {
        code: "env_file",
        state,
        text,
        path,
    }
}

fn check_settings(acc: &ClaudeAccountCfg, p: &Probe) -> Check {
    let patch = settings_patch(acc);
    let path = format!("{}/settings.json", p.dir);
    if !p.dir_exists {
        return Check {
            code: "settings",
            state: CheckState::Unknown,
            text: format!("chưa có {} — chưa kiểm settings.json", p.dir),
            path,
        };
    }
    let (state, text) = match &p.settings {
        SettingsState::Bad { path: f, why } => (
            CheckState::Fail,
            format!("{f}: {why} — sửa tay trước khi Lưu"),
        ),
        SettingsState::Missing(f) => {
            if patch.set.is_empty() {
                (CheckState::Ok, "không có khoá nào cần ghi".to_string())
            } else {
                (
                    CheckState::Fail,
                    format!("chưa có {f} — Lưu sẽ tạo ({} khoá)", patch.set.len()),
                )
            }
        }
        SettingsState::Found(cur) => {
            let conflicts = settings_conflicts(cur, &patch);
            let drift: Vec<String> = patch
                .set
                .iter()
                .filter(|s| json_at(cur, &s.path) != Some(&s.value))
                .map(|s| s.dotted())
                .collect();
            if !conflicts.is_empty() {
                (
                    CheckState::Fail,
                    format!(
                        "xung đột ở {} — chủ máy đã đặt giá trị khác; trang KHÔNG ghi đè",
                        conflicts.join(", ")
                    ),
                )
            } else if !drift.is_empty() {
                (
                    CheckState::Fail,
                    format!(
                        "lệch {} khoá ({}): bấm Lưu để đồng bộ",
                        drift.len(),
                        drift.join(", ")
                    ),
                )
            } else {
                let duong_dan = if patch.unsupported.is_empty() {
                    String::new()
                } else {
                    format!(" · {} trường chỉ đi qua dòng lệnh", patch.unsupported.len())
                };
                (
                    CheckState::Ok,
                    format!("khớp bản vá ({} khoá){duong_dan}", patch.set.len()),
                )
            }
        }
    };
    Check {
        code: "settings",
        state,
        text,
        path,
    }
}

fn check_launch(acc: &ClaudeAccountCfg) -> Check {
    let shadowed = structured_fields(acc);
    let (state, text) = match (legacy_line(acc), shadowed.is_empty()) {
        (Some(_), false) => (
            CheckState::Fail,
            format!(
                "`launch` viết tay ĐANG ĐÈ {} — sửa/xoá `launch` rồi Lưu",
                shadowed.join(", ")
            ),
        ),
        (Some(_), true) => (
            CheckState::Ok,
            "dùng `launch` viết tay (không có trường cấu trúc bị đè)".to_string(),
        ),
        (None, _) => (CheckState::Ok, "không có `launch` đè".to_string()),
    };
    Check {
        code: "launch",
        state,
        text,
        path: acc.name.clone(),
    }
}

fn check_live(p: &Probe) -> Check {
    let (state, text) = if let Some(why) = &p.blind {
        (
            CheckState::Unknown,
            format!("lượt này KHÔNG liệt kê được phiên: {why}"),
        )
    } else {
        match p.live_sessions {
            None => (CheckState::Unknown, "chưa đo phiên đang chạy".to_string()),
            Some(n) => (CheckState::Ok, format!("{n} phiên đang chạy")),
        }
    };
    Check {
        code: "live",
        state,
        text,
        path: String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn acc(name: &str) -> ClaudeAccountCfg {
        ClaudeAccountCfg {
            name: name.into(),
            ..Default::default()
        }
    }

    fn cfg() -> Config {
        Config {
            claude_cli: "claude".into(),
            ..Default::default()
        }
    }

    #[test]
    fn hang_khong_truong_cau_truc_giu_duong_cu_tung_byte() {
        let mut a = acc("acc4");
        a.config_dir = Some("~/.claude-acc4".into());
        let abs = expand_home(Path::new("~/.claude-acc4"));
        assert_eq!(
            launch_line(&cfg(), Some(&a)),
            format!("CLAUDE_CONFIG_DIR='{}' 'claude'", abs.display())
        );
        assert_eq!(launch_line(&cfg(), None), "'claude'");
        assert_eq!(launch_line_ps(&cfg(), None), "& 'claude'");
    }

    #[test]
    fn launch_legacy_thang_va_keu_len_khi_bi_che() {
        let mut a = acc("acc9");
        a.config_dir = Some("~/.claude-acc9".into());
        a.launch = Some("env $(cat $HOME/.opencode_acc9.env | xargs) claude".into());
        a.model = Some("deepseek-v4.1-flash".into());
        assert_eq!(
            effective_launch(&cfg(), Some(&a)),
            "env $(cat $HOME/.opencode_acc9.env | xargs) claude",
            "dòng viết tay phải thắng trong lúc di trú"
        );
        assert_eq!(structured_fields(&a), vec!["model"]);
    }

    #[test]
    fn dong_dung_du_base_url_env_va_model() {
        let mut a = acc("moi");
        a.config_dir = Some("~/.claude-moi".into());
        a.env_file = Some("~/.moi.env".into());
        a.base_url = Some("https://vd.ai/go/anthropic".into());
        a.model = Some("deepseek-v4.1-flash".into());
        a.extra_args = vec!["--debug".into()];
        a.extra_env
            .insert("CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS".into(), "1".into());
        assert_eq!(
            launch_line(&cfg(), Some(&a)),
            "env $(cat $HOME/.moi.env | xargs) \
             ANTHROPIC_BASE_URL=https://vd.ai/go/anthropic \
             CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS=1 \
             CLAUDE_CONFIG_DIR=$HOME/.claude-moi 'claude' --debug --model deepseek-v4.1-flash"
        );
    }

    #[test]
    fn api_key_as_doi_ten_bang_sed_va_khong_mang_gia_tri() {
        let mut a = acc("tam");
        a.env_file = Some("~/.tam.env".into());
        a.api_key_env = Some("OPENCODE_API_KEY".into());
        a.api_key_as = Some("ANTHROPIC_API_KEY".into());
        let line = launch_line(&cfg(), Some(&a));
        // Ghép biểu thức lúc chạy: cổng chung quét hình dạng `TÊN=giá-trị`, mà `s/^TÊN=/TÊN=/`
        // cũng mang hình dạng ấy dù không có giá trị nào.
        let sed_doi_ten = format!(
            "sed \"s/^{}=/{}=/\"",
            "OPENCODE_API_KEY", "ANTHROPIC_API_KEY"
        );
        assert!(line.contains(&sed_doi_ten), "{line}");
        assert!(
            !line.contains("sk-"),
            "giá trị khoá không được có mặt: {line}"
        );
    }

    #[test]
    fn env_for_child_bo_moi_ten_ket_thuc_key() {
        let mut a = acc("tam");
        a.base_url = Some("https://vd.ai".into());
        a.extra_env
            .insert("ANTHROPIC_API_KEY".into(), "sk-bimat".into());
        a.extra_env.insert("VUI".into(), "1".into());
        let env = env_for_child(&a);
        assert!(env.iter().all(|(_, v)| v.as_deref() != Some("sk-bimat")));
        assert!(env
            .iter()
            .any(|(k, v)| k == "VUI" && v.as_deref() == Some("1")));
        assert!(env
            .iter()
            .any(|(k, v)| k == "ANTHROPIC_BASE_URL" && v.as_deref() == Some("https://vd.ai")));
    }

    #[test]
    fn settings_patch_dung_hinh_va_khong_ghi_bi_mat() {
        let mut a = acc("tam");
        a.base_url = Some("https://vd.ai".into());
        a.api_key_env = Some("ANTHROPIC_API_KEY".into());
        a.model = Some("deepseek-v4.1-flash".into());
        a.effort = Some("high".into());
        a.extra_env.insert("MY_API_KEY".into(), "sk-bimat".into());
        let p = settings_patch(&a);
        assert_eq!(
            p.set,
            vec![
                Setting::new(&["env", "ANTHROPIC_BASE_URL"], json!("https://vd.ai")),
                Setting::new(&["apiKeyHelper"], json!("printenv ANTHROPIC_API_KEY")),
                Setting::new(&["model"], json!("deepseek-v4.1-flash")),
                Setting::new(
                    &["modelSettings", "deepseek-v4.1-flash", "effortLevel"],
                    json!("high")
                ),
            ]
        );
        assert!(p.unsupported.iter().any(|u| u.contains("MY_API_KEY")));
        assert!(!format!("{p:?}").contains("sk-bimat"));
    }

    #[test]
    fn xung_dot_chi_o_env_va_apikeyhelper() {
        let mut a = acc("tam");
        a.base_url = Some("https://moi.ai".into());
        a.model = Some("m2".into());
        let p = settings_patch(&a);
        let cur = json!({ "env": { "ANTHROPIC_BASE_URL": "https://cu.ai" }, "model": "m1" });
        assert_eq!(settings_conflicts(&cur, &p), vec!["env.ANTHROPIC_BASE_URL"]);
    }

    #[test]
    fn json_at_di_duoc_qua_ten_model_co_dau_cham() {
        let v = json!({ "modelSettings": { "deepseek-v4.1-flash": { "effortLevel": "high" } } });
        let path = ["modelSettings", "deepseek-v4.1-flash", "effortLevel"].map(String::from);
        assert_eq!(json_at(&v, &path), Some(&json!("high")));
    }

    #[test]
    fn validate_row_nhan_ten_hep_khong_nhan_dau_cach() {
        for ok in ["acc9", "acc_9", "a-b", "ABC"] {
            assert!(validate_row(ok).is_ok(), "{ok}");
        }
        for bad in ["mặc định", "", "a b", "../x", "a/b", "a.b"] {
            assert!(validate_row(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn ps_khong_nua_voi_env_file() {
        let mut a = acc("tam");
        a.env_file = Some("~/.tam.env".into());
        a.model = Some("m".into());
        let line = launch_line_ps(&cfg(), Some(&a));
        assert!(line.contains("& 'claude' --model 'm'"), "{line}");
        assert!(
            !line.contains(".tam.env"),
            "env_file không được giả vờ đã nạp: {line}"
        );
    }

    // ── GĐ 2 — chẩn đoán ──────────────────────────────────────────────────

    fn quota_mau(ten: &str) -> Quota {
        Quota {
            account: ten.into(),
            week_pct: Some(10),
            week_resets_at: None,
            hour5_pct: None,
            hour5_resets_at: None,
            fetched_at_ms: None,
            why_unknown: None,
            chua_dung_duoc: None,
            models: Vec::new(),
            model_use: None,
        }
    }

    fn tim<'a>(c: &'a [Check], code: &str) -> &'a Check {
        c.iter()
            .find(|x| x.code == code)
            .unwrap_or_else(|| panic!("không có mục {code}"))
    }

    #[test]
    fn chan_doan_chi_ra_tung_cho_hong() {
        let mut a = acc("acc9");
        a.config_dir = Some("~/.claude-acc9".into());
        a.env_file = Some("~/.acc9.env".into());
        a.api_key_env = Some("ANTHROPIC_API_KEY".into());
        a.model = Some("m9".into());
        let p = Probe {
            dir: "/tmp/.claude-acc9".into(),
            dir_exists: true,
            projects: ProjectsState::RealDir,
            env_file: EnvFileState::Found {
                path: "~/.acc9.env".into(),
                mode: Some(0o644),
                vars: vec!["BIEN_KHAC".into()],
            },
            settings: SettingsState::Found(json!({ "model": "m1" })),
            quota: Some(quota_mau("acc9")),
            api_key: None,
            live_sessions: Some(0),
            blind: None,
        };
        let c = checks(&a, &p);
        assert_eq!(c.len(), 8, "8 mục, không hơn không kém");
        assert_eq!(tim(&c, "dir").state, CheckState::Ok);
        assert_eq!(tim(&c, "projects").state, CheckState::Fail, "thư mục THẬT");
        assert_eq!(tim(&c, "login").state, CheckState::Ok);
        assert_eq!(
            tim(&c, "api_key").state,
            CheckState::Fail,
            "khai api_key_env mà settings không có helper"
        );
        assert!(tim(&c, "api_key")
            .text
            .contains("printenv ANTHROPIC_API_KEY"));
        assert_eq!(tim(&c, "env_file").state, CheckState::Fail);
        assert!(
            tim(&c, "env_file").text.contains("600"),
            "{:?}",
            tim(&c, "env_file")
        );
        assert!(tim(&c, "env_file").text.contains("ANTHROPIC_API_KEY"));
        assert_eq!(tim(&c, "settings").state, CheckState::Fail, "lệch model");
        assert_eq!(tim(&c, "launch").state, CheckState::Ok);
        assert_eq!(tim(&c, "live").state, CheckState::Ok);
        assert_eq!(problems(&c).len(), 4);

        // Vế ngược — cùng hàng, sửa đúng 4 chỗ thì 8/8 xanh. Một bộ chẩn đoán
        // không đổi được màu là một phép đo mù.
        let p2 = Probe {
            projects: ProjectsState::Link(home_dir().unwrap().join(".claude").join("projects")),
            env_file: EnvFileState::Found {
                path: "~/.acc9.env".into(),
                mode: Some(0o600),
                vars: vec!["ANTHROPIC_API_KEY".into()],
            },
            settings: SettingsState::Found(json!({
                "model": "m9",
                "apiKeyHelper": "printenv ANTHROPIC_API_KEY"
            })),
            api_key: Some("API key …WXYZ".into()),
            ..p
        };
        let c2 = checks(&a, &p2);
        assert_eq!(problems(&c2).len(), 0, "{c2:?}");
    }

    #[test]
    fn khong_do_duoc_la_trang_thai_rieng_khong_phai_hong() {
        let a = acc("acc9");
        let mut q = quota_mau("acc9");
        q.why_unknown = Some("không đọc được /x/.claude.json".into());
        let p = Probe {
            dir: "/tmp/x".into(),
            dir_exists: true,
            quota: Some(q),
            blind: Some("đọc `claude agents` hỏng".into()),
            ..Default::default()
        };
        let c = checks(&a, &p);
        assert_eq!(
            tim(&c, "login").state,
            CheckState::Unknown,
            "{:?}",
            tim(&c, "login")
        );
        assert_eq!(tim(&c, "live").state, CheckState::Unknown);
        // Không đo được KHÔNG bị đếm là hỏng — nếu gộp, đây là một lỗi bịa ra.
        assert!(c
            .iter()
            .filter(|x| x.code == "login" || x.code == "live")
            .all(|x| x.state == CheckState::Unknown));
    }

    #[test]
    fn dang_nhap_hong_thi_chep_nguyen_van_ly_do_cua_so_han_muc() {
        let a = acc("acc9");
        let mut q = quota_mau("acc9");
        q.chua_dung_duoc = Some("chưa đăng nhập (chưa có sổ .claude.json)".into());
        let p = Probe {
            dir: "/tmp/x".into(),
            dir_exists: true,
            quota: Some(q),
            live_sessions: Some(1),
            ..Default::default()
        };
        let c = checks(&a, &p);
        let login = tim(&c, "login");
        assert_eq!(login.state, CheckState::Fail);
        assert_eq!(login.text, "chưa đăng nhập (chưa có sổ .claude.json)");
        assert!(
            !login.text.contains('0'),
            "không được bịa 0%: {}",
            login.text
        );
    }

    #[test]
    fn tai_khoan_api_key_khong_can_dang_nhap_goi() {
        // acc9 chạy `apiKeyHelper`: sổ không có `oauthAccount` là ĐÚNG, không phải hỏng.
        let a = acc("acc9");
        let mut q = quota_mau("acc9");
        q.chua_dung_duoc = Some("chưa đăng nhập (sổ không có oauthAccount)".into());
        let p = Probe {
            dir: "/tmp/x".into(),
            dir_exists: true,
            quota: Some(q),
            api_key: Some("API key …WXYZ".into()),
            ..Default::default()
        };
        let login = tim(&checks(&a, &p), "login").clone();
        assert_eq!(login.state, CheckState::Ok, "{login:?}");
        assert!(login.text.contains("API key …WXYZ"), "{}", login.text);
        assert!(!login.text.contains("oauthAccount"), "{}", login.text);

        // Vế ngược — cùng sổ, KHÔNG có key: vẫn ✗ nguyên văn như cũ.
        let p2 = Probe { api_key: None, ..p };
        let login2 = tim(&checks(&a, &p2), "login").clone();
        assert_eq!(login2.state, CheckState::Fail, "{login2:?}");
        assert_eq!(login2.text, "chưa đăng nhập (sổ không có oauthAccount)");
    }

    /// Có API key là đủ để `login` xanh — kể cả khi sổ hạn mức CHƯA đo. Vế
    /// ngược: không key + chưa đo vẫn là ⚠ (Unknown), không thành ✓.
    #[test]
    fn api_key_du_de_login_xanh_ca_khi_chua_do_so_han_muc() {
        let a = acc("acc9");
        let p = Probe {
            dir: "/tmp/x".into(),
            dir_exists: true,
            quota: None,
            api_key: Some("API key …WXYZ".into()),
            ..Default::default()
        };
        assert_eq!(tim(&checks(&a, &p), "login").state, CheckState::Ok);
        let p2 = Probe { api_key: None, ..p };
        let login2 = tim(&checks(&a, &p2), "login").clone();
        assert_eq!(login2.state, CheckState::Unknown, "{login2:?}");
        assert!(login2.text.contains("chưa đo sổ hạn mức"), "{login2:?}");
    }

    #[test]
    fn launch_de_truong_cau_truc_la_mot_muc_hong() {
        let mut a = acc("acc9");
        a.launch = Some("env $(cat ~/.acc9.env | xargs) claude".into());
        a.model = Some("m9".into());
        let c = checks(&a, &Probe::default());
        assert_eq!(tim(&c, "launch").state, CheckState::Fail);
        assert!(tim(&c, "launch").text.contains("model"));
    }

    #[cfg(unix)]
    #[test]
    fn probe_doc_dia_that_va_doi_chung_nguoc() {
        use std::os::unix::fs::PermissionsExt;

        let goc = std::env::temp_dir().join(format!("huba-probe-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&goc);
        let dir = goc.join(".claude-thu");
        std::fs::create_dir_all(&dir).unwrap();
        let _ = std::os::unix::fs::symlink(
            home_dir().unwrap().join(".claude").join("projects"),
            dir.join("projects"),
        );
        let envf = goc.join("thu.env");
        // Fixture GIẢ: ghép tên biến lúc chạy để cổng chung không chấm ĐỎ OAN hình dạng
        // `TÊN=giá-trị` của một giá trị THỬ. Test ngay dưới assert nó KHÔNG lọt vào Probe.
        std::fs::write(
            &envf,
            format!(
                "ANTHROPIC_API_KEY={}\n# ghi chú\nKY_KHONG_SAO=1\n",
                "sk-khong-duoc-lo"
            ),
        )
        .unwrap();
        std::fs::set_permissions(&envf, std::fs::Permissions::from_mode(0o600)).unwrap();
        std::fs::write(dir.join("settings.json"), r#"{"model":"cu"}"#).unwrap();

        let mut a = acc("thu");
        a.config_dir = Some(dir.display().to_string());
        a.env_file = Some(envf.display().to_string());
        a.api_key_env = Some("ANTHROPIC_API_KEY".into());
        a.model = Some("moi".into());

        let mut p = probe(&a, Some(quota_mau("thu")));
        p.live_sessions = Some(2);
        assert!(p.dir_exists);
        assert!(
            matches!(p.projects, ProjectsState::Link(_)),
            "{:?}",
            p.projects
        );
        match &p.env_file {
            EnvFileState::Found { mode, vars, .. } => {
                assert_eq!(*mode, Some(0o600));
                assert_eq!(
                    vars,
                    &vec!["ANTHROPIC_API_KEY".to_string(), "KY_KHONG_SAO".to_string()]
                );
            }
            other => panic!("{other:?}"),
        }
        assert!(
            !format!("{p:?}").contains("sk-khong-duoc-lo"),
            "Probe không được mang giá trị bí mật"
        );
        let c = checks(&a, &p);
        assert_eq!(tim(&c, "projects").state, CheckState::Ok);
        assert_eq!(tim(&c, "env_file").state, CheckState::Ok);
        assert_eq!(tim(&c, "settings").state, CheckState::Fail, "model lệch");

        // Đối chứng ngược ①: `projects` thành thư mục THẬT ⟹ ĐỎ.
        std::fs::remove_file(dir.join("projects")).unwrap();
        std::fs::create_dir(dir.join("projects")).unwrap();
        assert_eq!(probe(&a, None).projects, ProjectsState::RealDir);

        // Đối chứng ngược ②: quyền 644 ⟹ mục tệp env ĐỎ, nêu đúng số.
        std::fs::set_permissions(&envf, std::fs::Permissions::from_mode(0o644)).unwrap();
        let c2 = checks(&a, &probe(&a, None));
        assert_eq!(tim(&c2, "env_file").state, CheckState::Fail);
        assert!(tim(&c2, "env_file").text.contains("644"));

        let _ = std::fs::remove_dir_all(&goc);
    }

    /// Một policy, hai đầu ra: dòng launch của opencode và bức tường quyền.
    #[test]
    fn opencode_launch_va_tuong_dung_mot_policy() {
        let mut a = acc("acc8");
        a.cli = Some("opencode".into());
        a.env_file = Some("~/.opencode_acc8.env".into());
        a.api_key_env = Some("OPENCODE_API_KEY".into());
        a.model = Some("opencode-go/deepseek-v4.1-flash".into());
        a.opencode_config = Some(json!({"plugin":["/x/plugin"],"provider":{"p":{"x":1}}}));
        let line = launch_line(&cfg(), Some(&a));
        assert!(
            line.contains("XDG_DATA_HOME=$HOME/.opencode-acc8/data"),
            "{line}"
        );
        assert!(line.contains("OPENCODE_CONFIG_CONTENT="), "{line}");
        assert!(!line.contains("--model"), "{line}");
        assert!(!line.contains("CLAUDE_CONFIG_DIR"), "{line}");
        assert!(!line.contains("ANTHROPIC_BASE_URL"), "{line}");
        let c = opencode_config_content(&a);
        assert_eq!(
            c.pointer("/model"),
            Some(&json!("opencode-go/deepseek-v4.1-flash")),
            "model phải đi qua OPENCODE_CONFIG_CONTENT"
        );
        assert_eq!(c.pointer("/permission/bash/*deploy*"), Some(&json!("deny")));
        assert_eq!(c.pointer("/permission/webfetch"), Some(&json!("deny")));
        assert_eq!(c.pointer("/plugin/0"), Some(&json!("/x/plugin")));
        assert!(c.pointer("/agent/huba-fork").is_some());
        let ext = c
            .pointer("/permission/external_directory")
            .and_then(Value::as_object)
            .expect("external_directory phải có mặt");
        assert!(
            ext.keys().any(|k| k.ends_with("/projects/*")),
            "phải cho cây workspace: {ext:?}"
        );
        assert_eq!(ext.get("/tmp/*"), Some(&json!("allow")));
        let claude = acc("acc3");
        assert!(!claude.is_opencode());
        assert!(!launch_line(&cfg(), Some(&claude)).contains("OPENCODE"));
    }
}

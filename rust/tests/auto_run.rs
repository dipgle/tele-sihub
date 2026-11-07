//! Cổng của cỗ máy TỰ CHẠY lệnh — nay là một cái DẤU, không phải phép đoán.
//!
//! 🔴 Hà 2026-08-24, hỏi *"Tại sao lại cần allow làm gì vậy?"* rồi
//! *"có quy tắc nào để bảo claude đánh dấu vào output kết quả là lệnh để bắt
//! tôi chạy không"*, rồi chốt **mức 2: chỉ dấu, bỏ allow**.
//!
//! Bản 23/08 đoán theo hình dạng (`commands_in_report`) rồi phải tự chặn lại
//! thứ chính nó đoán bừa bằng một danh sách cho phép — và cái danh sách ấy đã
//! có một lỗ RCE (khớp tiền tố ⟹ `bash ./gate.sh && rm -rf ~` lọt).
//!
//! Nay nguồn đổi hẳn: chỉ chạy dòng phiên **CỐ Ý đánh dấu**. Hết đoán thì hết
//! thứ phải chặn — nên danh sách cho phép đi luôn, có chủ ý, xem bia mộ trong
//! `pipeline.rs`.
//!
//! ⚠ Cái dấu nói *"mô hình cố ý bảo chạy"*, KHÔNG nói *"chủ máy cho phép"*. Hà
//! biết rủi ro ấy và chọn thế.

use huba::keys::{marked_commands, RUN_MARK};

#[test]
fn a_marked_line_is_picked_up() {
    let text = format!("Anh chạy hộ tôi:\n{RUN_MARK}\ncd ~/projects/huba/rust && cargo test --offline\nXong báo tôi.");
    assert_eq!(
        marked_commands(&text, 8),
        vec!["cd ~/projects/huba/rust && cargo test --offline"]
    );
}

/// 🔴 HÀNG RÀO CHÍNH: không có dấu thì KHÔNG chạy, dù trông giống lệnh tới đâu.
///
/// Đây là toàn bộ khác biệt giữa hàm này và `commands_in_report`. Nới chỗ này
/// là quay về đúng bản đã có lỗ.
#[test]
fn an_unmarked_command_is_never_run() {
    for text in [
        "cd ~/projects/huba && cargo test --offline",
        "git -C ~/projects/huba push origin main",
        "Chạy: bash ~/projects/huba/install_update.sh",
        "rm -rf ~",
    ] {
        assert!(
            marked_commands(text, 8).is_empty(),
            "dòng KHÔNG đánh dấu mà vẫn chạy: {text:?}"
        );
    }
}

/// 🔴 Dấu phải CHIẾM TRỌN một dòng.
///
/// Nếu chỉ cần "có chứa" thì một câu văn nhắc tới cái dấu — như chính tài liệu
/// này, hay một tin nhắn giải thích quy ước cho người dùng — sẽ tự kích hoạt.
/// Và `last_text` còn mang cả chữ phiên trích từ web, từ diff, từ tệp.
#[test]
fn the_mark_must_own_its_whole_line() {
    for text in [
        format!("echo \"{RUN_MARK}\"\nrm -rf ~"),
        format!("Quy ước là {RUN_MARK} đặt trên dòng lệnh\nrm -rf ~"),
        format!("{RUN_MARK} rm -rf ~"),
        format!("# {RUN_MARK}\nrm -rf ~"),
    ] {
        assert!(
            marked_commands(&text, 8).is_empty(),
            "dấu nấp trong một dòng khác mà vẫn kích hoạt: {text:?}"
        );
    }
}

/// Một dấu ăn ĐÚNG một lệnh — không mở cổng cho cả phần còn lại của tin.
#[test]
fn one_mark_arms_exactly_one_line() {
    let text = format!("{RUN_MARK}\nls ~/projects\nrm -rf ~\ngit push --force");
    assert_eq!(marked_commands(&text, 8), vec!["ls ~/projects"]);
}

/// Dòng trống giữa dấu và lệnh không làm mất dấu — TUI hay chèn một dòng trống.
#[test]
fn a_blank_line_after_the_mark_is_tolerated() {
    let text = format!("{RUN_MARK}\n\n   \nls ~/projects\n");
    assert_eq!(marked_commands(&text, 8), vec!["ls ~/projects"]);
}

/// Ghép được với bản vá nối `\` sáng nay: lệnh nhiều dòng đi nguyên khối.
#[test]
fn a_marked_multi_line_command_arrives_whole() {
    let text =
        format!("{RUN_MARK}\ncd ~/projects/huba && \\\ncargo test --offline && \\\necho XONG\n");
    let got = marked_commands(&text, 8);
    assert_eq!(got.len(), 1, "{got:#?}");
    assert!(got[0].starts_with("cd ~/projects/huba &&"), "{got:#?}");
    assert!(got[0].contains("echo XONG"), "mất đuôi khối: {got:#?}");
    assert!(!got[0].contains('\\'), "còn dấu nối: {got:#?}");
}

/// Dấu nhắc của TUI đứng trước lệnh thì bóc, cùng bộ với `commands_in_report`.
#[test]
fn tui_prompt_decoration_is_stripped() {
    let text = format!("{RUN_MARK}\n  $ ls ~/projects\n");
    assert_eq!(marked_commands(&text, 8), vec!["ls ~/projects"]);
}

/// Nhiều lệnh đánh dấu thì lấy đủ, theo đúng thứ tự, tới trần `max`.
#[test]
fn several_marks_keep_their_order_and_respect_the_cap() {
    let text = format!("{RUN_MARK}\nls a\nvăn xuôi\n{RUN_MARK}\nls b\n{RUN_MARK}\nls c\n");
    assert_eq!(marked_commands(&text, 8), vec!["ls a", "ls b", "ls c"]);
    assert_eq!(marked_commands(&text, 2), vec!["ls a", "ls b"]);
}

/// Cờ tắt vẫn phải tắt được cả cỗ máy.
#[test]
fn the_feature_can_still_be_switched_off() {
    let cfg = huba::config::Config::default();
    assert!(cfg.auto_run.enabled, "mặc định bật — Hà chốt mức 2");
}

// ───────── MỘT yêu cầu, HAI đường nhận: phải chạy ĐÚNG MỘT lần ─────────
//
// 🔴 Ca thật, đo được (Quang 07/11): phiên `[ido-pm/d23]` làm đúng quy ước —
// ghi `huba-run.txt` VÀ in dấu `#huba-run` trong câu trả lời. Hòm thư nhận
// (log `runin_inbox_queued`, viec 2924) rồi `auto_run` đọc dấu ấy trên màn và
// bắn thêm lượt nữa; nhật ký có HAI `long_job_started` (n=58 pid 88490 · n=59
// pid 88491) cùng một dòng lệnh, cùng một phiên, cách nhau 6 ms. Một lệnh
// `day-goi.sh` vì thế chạy hai lần song song.
//
// Cổng nay là MỘT cuốn sổ chung: hòm thư nhận là ghi dấu, `auto_run` đọc cuốn
// ấy trước khi bắn. Ba bài dưới đo vế thuận, vế ngược (dòng KHÁC vẫn phải
// chạy), và vế "cổng đỏ được" (bỏ dấu đi thì chính dòng ấy bắn lại).

use huba::db::Db;
use huba::pipeline::{doc_so_da_chay, ghi_da_chay, lenh_danh_dau_chua_chay, quick_token};

/// Dòng lệnh nguyên văn trong ca thật — dùng lại để bài kiểm này đọc ra đúng
/// thứ nhật ký đã ghi (đường dẫn nhà rút về `~/` cho kho công khai).
const LENH_CA_THAT: &str =
    "cd ~/projects/ido-pm/.tmp/wt-d23-p2 && bash scripts/day-goi.sh ido-pm-20261107-183000";
const PHIEN_CA_THAT: &str = "c2c1d42c-ab84-4d56-a184-8ac039c76a34";

fn man_cua_phien(line: &str) -> String {
    format!("Tôi đã xếp việc:\n{RUN_MARK}\n{line}\nXong báo tôi.")
}

fn mo_so() -> (tempfile::TempDir, Db) {
    let dir = tempfile::tempdir().unwrap();
    let db = Db::open(&dir.path().join("huba.sqlite")).expect("mở db");
    (dir, db)
}

#[test]
fn ho_thu_da_nhan_thi_dau_tren_man_khong_ban_lai() {
    let (_dir, db) = mo_so();
    // Màn CÓ dấu + đúng dòng ấy: chưa ai ghi sổ thì `auto_run` sẽ bắn.
    let man = man_cua_phien(LENH_CA_THAT);
    let chua_co_gi: Vec<String> = Vec::new();
    assert_eq!(
        lenh_danh_dau_chua_chay(&man, PHIEN_CA_THAT, &chua_co_gi),
        Some(LENH_CA_THAT.to_string()),
        "điều kiện nền: chưa ghi sổ thì dấu trên màn PHẢI bắn được"
    );

    // Hòm thư nhận đúng dòng ấy (đây là việc `runin_inbox_tick` làm).
    ghi_da_chay(&db, PHIEN_CA_THAT, LENH_CA_THAT);
    let so = doc_so_da_chay(&db);
    let seen = so.get(PHIEN_CA_THAT).cloned().unwrap_or_default();
    assert_eq!(seen, vec![quick_token(PHIEN_CA_THAT, LENH_CA_THAT)]);

    assert_eq!(
        lenh_danh_dau_chua_chay(&man, PHIEN_CA_THAT, &seen),
        None,
        "hòm thư đã nhận mà dấu trên màn vẫn bắn ⟹ chạy HAI lần (ca 07/11)"
    );
}

/// Vế ngược của cùng cuốn sổ: chặn theo DÒNG, không theo phiên. Một dòng khác
/// trên cùng màn vẫn phải chạy — nếu không thì "sửa lỗi chạy hai lần" đã đổi
/// thành "phiên ấy không bao giờ tự chạy được nữa", một lỗi im lặng hơn.
#[test]
fn dong_khac_tren_cung_man_van_chay() {
    let (_dir, db) = mo_so();
    let dong_khac = "cargo test --offline";
    ghi_da_chay(&db, PHIEN_CA_THAT, LENH_CA_THAT);
    let seen = doc_so_da_chay(&db)
        .get(PHIEN_CA_THAT)
        .cloned()
        .unwrap_or_default();
    let man = man_cua_phien(dong_khac);
    assert_eq!(
        lenh_danh_dau_chua_chay(&man, PHIEN_CA_THAT, &seen),
        Some(dong_khac.to_string())
    );
    // …và phiên KHÁC với cùng dòng cũng không bị chặn oan.
    assert_eq!(
        lenh_danh_dau_chua_chay(&man_cua_phien(LENH_CA_THAT), "phiên-khác", &seen),
        Some(LENH_CA_THAT.to_string())
    );
}

/// 🔴 Cổng phải ĐỎ được: gỡ đúng một dòng (`ghi_da_chay` của hòm thư) là bài
/// trên phải đỏ. Bài này chạy chính phép đo ấy trên một cuốn sổ RỖNG — tức
/// chứng minh nó không xanh vì lý do nào khác.
#[test]
fn doi_chung_nguoc_bo_dau_ghi_so_thi_chay_lai() {
    let (_dir, db) = mo_so();
    let man = man_cua_phien(LENH_CA_THAT);
    // KHÔNG gọi `ghi_da_chay` — mô phỏng bản chưa vá.
    let so = doc_so_da_chay(&db);
    let seen = so.get(PHIEN_CA_THAT).cloned().unwrap_or_default();
    assert_eq!(
        lenh_danh_dau_chua_chay(&man, PHIEN_CA_THAT, &seen),
        Some(LENH_CA_THAT.to_string()),
        "cuốn sổ rỗng mà không bắn ⟹ cổng MÙ, bài chống-chạy-lặp ở trên xanh oan"
    );
}

/// Đếm số lời gọi `ghi_da_chay` trong mã nguồn — tách thành hàm thuần để bài
/// cuối đối chứng ngược được (một hàm luôn trả 0 cũng làm tệp này xanh).
fn dem_loi_goi(src: &str) -> usize {
    src.matches("ghi_da_chay(db, &sid, &cmd)").count()
}

/// Hòm thư có HAI nhánh nhận (có sổ việc / đường cũ) — thiếu một nhánh là
/// đúng nửa số ca chạy hai lần quay lại, và không nhìn thấy được từ log.
#[test]
fn ca_hai_nhanh_ho_thu_deu_ghi_so() {
    let src = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/pipeline.rs"),
    )
    .expect("KHÔNG ĐO ĐƯỢC: không đọc nổi src/pipeline.rs");
    assert!(
        src.len() > 500_000,
        "KHÔNG ĐO ĐƯỢC: pipeline.rs chỉ {} byte — sai tệp",
        src.len()
    );
    assert_eq!(
        dem_loi_goi(&src),
        2,
        "hai nhánh nhận hòm thư trong `runin_inbox_tick` phải cùng ghi sổ \
         chống-chạy-lặp; thiếu một nhánh ⟹ nửa số ca chạy hai lần quay lại"
    );
    // Đối chứng ngược: bộ đếm phải thấy dòng cấy.
    assert_eq!(dem_loi_goi("… ghi_da_chay(db, &sid, &cmd); …"), 1);
    assert_eq!(dem_loi_goi("ghi_da_chay(db, sid, cmd)"), 0);
}

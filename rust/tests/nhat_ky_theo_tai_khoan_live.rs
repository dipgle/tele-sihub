//! THĂM DÒ trên máy thật (chỉ đọc): mọi nhật ký nằm dưới `projects` RIÊNG của
//! một tài khoản — huba có tìm ra theo id không, đường CŨ (một gốc chung) và
//! đường MỚI (`find_transcript_any`) đếm cạnh nhau.
//!
//! `cargo test --offline --test nhat_ky_theo_tai_khoan_live -- --ignored --nocapture`
//!
//! Mẫu số là MỌI tệp `<gốc tài khoản>/projects/*/*.jsonl` không đi qua liên kết
//! về gốc chung; đường mới hụt dù chỉ một tệp ⟹ đỏ. Không có tài khoản nào mang
//! `projects` riêng ⟹ KHÔNG ĐO ĐƯỢC (panic riêng), không phải xanh.

use std::path::PathBuf;

use huba::sessions;

#[test]
#[ignore]
fn moi_nhat_ky_o_projects_rieng_deu_tim_duoc() {
    let cfg = huba::config::load(None).expect("đọc huba.config.json");
    let chung =
        std::fs::canonicalize(cfg.claude_transcript_root().join("projects")).expect("gốc chung");
    let mut mau = 0usize;
    let (mut cu, mut moi) = (0usize, 0usize);
    let mut hut: Vec<PathBuf> = Vec::new();
    for root in sessions::transcript_roots(&cfg) {
        let projects = root.join("projects");
        let Ok(that) = std::fs::canonicalize(&projects) else {
            continue;
        };
        if that == chung {
            continue;
        }
        for dir in std::fs::read_dir(&projects).unwrap().flatten() {
            for f in std::fs::read_dir(dir.path())
                .into_iter()
                .flatten()
                .flatten()
            {
                let p = f.path();
                if p.extension().and_then(|x| x.to_str()) != Some("jsonl") {
                    continue;
                }
                let id = p.file_stem().unwrap().to_string_lossy().into_owned();
                mau += 1;
                if sessions::find_transcript(&cfg.claude_transcript_root(), &id).is_some() {
                    cu += 1;
                }
                match sessions::find_transcript_any(&cfg, &id) {
                    Some(_) => moi += 1,
                    None => hut.push(p),
                }
            }
        }
        println!("gốc {} ⟹ mẫu số dồn {mau}", root.display());
    }
    println!(
        "MAU_SO={mau} CU_TIM_DUOC={cu} MOI_TIM_DUOC={moi} MOI_HUT={}",
        hut.len()
    );
    assert!(
        mau > 0,
        "KHÔNG ĐO ĐƯỢC: không tài khoản nào có projects riêng"
    );
    assert!(hut.is_empty(), "đường mới hụt: {hut:?}");
}

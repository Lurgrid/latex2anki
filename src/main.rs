use genanki_rs::{basic_model, Deck, Note, Package};
use rand::Rng;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, fs};

#[derive(Debug)]
struct QuesRes {
    q: Vec<String>,
    r: Vec<String>,
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("*** Syntax: {} FILE", args[0]);
        return;
    }
    let lines: Vec<String> = read_lines(&args[1]);

    let mut header: Vec<String> = vec![];
    let mut qrs: Vec<QuesRes> = vec![];
    let mut i = 0;

    while i < lines.len() && !lines[i].contains(r#"\begin{document}"#) {
        header.push(lines[i].to_owned());
        i += 1;
    }

    header.push("\\pagenumbering{gobble}".to_string());

    while i < lines.len() {
        while i < lines.len() && !lines[i].contains("%Q") {
            i += 1;
        }
        if i == lines.len() {
            break;
        }
        i += 1;
        let mut q: QuesRes = QuesRes {
            q: vec![],
            r: vec![],
        };
        while i < lines.len() && !lines[i].contains("%R") {
            q.q.push(lines[i].to_owned());
            i += 1;
        }
        if i == lines.len() {
            break;
        }
        i += 1;
        while i < lines.len() && !lines[i].contains("%F") {
            q.r.push(lines[i].to_owned());
            i += 1;
        }
        if i == lines.len() {
            break;
        }
        qrs.push(q);
        i += 1;
    }

    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
        .to_string();

    for (i, qr) in qrs.iter().enumerate() {
        let q = String::from("q") + &(i.to_string()) + &since_the_epoch;
        let r = String::from("r") + &(i.to_string()) + &since_the_epoch;
        fs::write(
            q.to_owned() + ".tex",
            format!(
                "{}\\begin{{document}}\n{}\n\\end{{document}}",
                header.join("\n"),
                qr.q.join("\n")
            ),
        )
        .unwrap();
        fs::write(
            r.to_owned() + ".tex",
            format!(
                "{}\\begin{{document}}\n{}\n\\end{{document}}",
                header.join("\n"),
                qr.r.join("\n")
            ),
        )
        .unwrap();
        latex_to_pdf(&q);
        latex_to_pdf(&r);
        let _ = fs::remove_file(q.to_owned() + ".tex");
        let _ = fs::remove_file(r.to_owned() + ".tex");
    }

    let mut rng = rand::thread_rng();

    let mut deck = Deck::new(
        rng.gen::<i64>(),
        "Generate Deck",
        "Generate Deck using latex2anki",
    );

    let mut files: Vec<String> = vec![];

    for i in 0..qrs.len() {
        deck.add_note(
            Note::new(
                basic_model(),
                vec![
                    format!("<img src=\"q{}{}.jpg\">", i, since_the_epoch).as_str(),
                    format!("<img src=\"r{}{}.jpg\">", i, since_the_epoch).as_str(),
                ],
            )
            .unwrap(),
        );
        files.push(format!("q{}{}.jpg", i, since_the_epoch));
        files.push(format!("r{}{}.jpg", i, since_the_epoch));
    }
    let mut my_package =
        Package::new(vec![deck], files.iter().map(|s| s.as_str()).collect()).unwrap();
    my_package.write_to_file("output.apkg").unwrap();

    for f in files {
        let _ = fs::remove_file(f);
    }
}

fn latex_to_pdf(file: &str) {
    let tex = file.to_owned() + ".tex";
    let out = Command::new("pdflatex")
        .arg(&tex)
        .output()
        .expect("failed to execute process");
    if !out.status.success() {
        println!(
            "{}",
            String::from_utf8(out.stdout).unwrap_or("Error".to_string())
        );
        return;
    }

    let cpdf = file.to_owned() + "-cropped.pdf";
    let pdf = file.to_owned() + ".pdf";

    let out = Command::new("pdfcrop")
        .arg(&pdf)
        .arg(&cpdf)
        .output()
        .expect("failed to execute process");
    if !out.status.success() {
        println!(
            "{}",
            String::from_utf8(out.stderr).unwrap_or("Error".to_string())
        );
        return;
    }

    let out = Command::new("pdftoppm")
        .args(["-jpeg", "-singlefile", &cpdf, &file])
        .output()
        .expect("failed to execute process");

    if !out.status.success() {
        println!(
            "{}",
            String::from_utf8(out.stderr).unwrap_or("Error".to_string())
        );
        return;
    }

    let _ = fs::remove_file(file.to_owned() + ".aux");
    let _ = fs::remove_file(file.to_owned() + ".log");
    let _ = fs::remove_file(file.to_owned() + ".out");
    let _ = fs::remove_file(pdf);
    let _ = fs::remove_file(cpdf);
    let _ = fs::remove_file(tex);
}

fn read_lines(filename: &str) -> Vec<String> {
    fs::read_to_string(filename)
        .expect("Cannot open the file")
        .lines()
        .map(String::from)
        .collect()
}

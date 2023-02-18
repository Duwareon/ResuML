use genpdf::{elements::*, style::*, *};
use std::{env, fs};

pub fn main() {
    // Declare variables for input text
    let args: Vec<String> = env::args().filter(|x| !x.starts_with('-')).collect();
    if args.len() <= 1 {
        panic!("Please input a ResuMarkup file")
    }
    let flags: Vec<String> = env::args().filter(|x| x.starts_with('-')).collect();
    let filepath: String = args[1].clone();
    let sourcefile = fs::read_to_string(filepath).expect("Cannot open file");
    let sourcetext: Vec<&str> = sourcefile.split("\n").collect();
    let mut textstack = sourcetext.clone();

    textstack.reverse();

    // Parse flags
    for i in flags {
        println!("Option not recognized: {i}")
    }

    // Declare variables for pdf
    let font = fonts::from_files("/usr/share/fonts/vollkorn/", "Vollkorn", None)
        .expect("Failed to load font");

    let mut doc = Document::new(font);

    // Set PDF options
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(30);
    doc.set_page_decorator(decorator);
    doc.set_font_size(15);

    // Write text to PDF
    while !textstack.is_empty() {
        let line = textstack.pop().unwrap();

        if line.starts_with("#+") {
            let splitline: Vec<&str> = line.split(':').collect();
            let linestart = splitline[0];

            match linestart {
                // Write applicant name as title
                "#+AUTHOR" => {
                    let parstyle: Style = Style::default().with_font_size(28).bold();
                    doc.push(
                        Paragraph::default()
                            .styled_string(splitline[1], parstyle)
                            .aligned(Alignment::Center),
                    );
                    doc.push(
                        Paragraph::default()
                            .styled_string("---------------------------------", parstyle)
                            .aligned(Alignment::Center),
                    );
                }

                "#+SUBTITLE" => {
                    let parstyle: Style = Style::default().with_font_size(20).italic();
                    doc.push(
                        Paragraph::default()
                            .styled_string(splitline[1], parstyle)
                            .aligned(Alignment::Center),
                    )
                }

                // Write text with bullet point
                "#+POINT" => {
                    doc.push(BulletPoint::new(Paragraph::new(splitline[1])).with_bullet("-"));
                }

                // Write section title for an experience entry
                "#+EXPERIENCE" => {
                    let parstyle: Style = Style::default().with_font_size(16);
                    doc.push(Paragraph::default().styled_string(splitline[1], parstyle))
                }

                // Write section subtitle for experience entry
                // TODO: Figure out a better word than specialization
                "#+SPECIALIZATION" => {
                    let parstyle: Style = Style::default().italic();
                    doc.push(Paragraph::default().styled_string(splitline[1], parstyle))
                }

                "#+START" => {
                    let parstyle: Style = Style::default();

                    if !textstack.is_empty() {
                        let start = splitline[1];
                        let nextline = textstack.pop().unwrap();
                        if nextline.starts_with("#+END:") {
                            let endsplit: Vec<&str> = nextline.split(':').collect();
                            let end = endsplit[1];
                            doc.push(
                                Paragraph::default()
                                    .styled_string(format!("{start} - {end}"), parstyle),
                            )
                        } else {
                            doc.push(
                                Paragraph::default()
                                    .styled_string(format!("{start} - Present"), parstyle),
                            )
                        }
                    }
                }

                // Set section of resume (e.g. Education, Experience, Skills)
                "#+STARTSECTION" => {
                    let parstyle: Style = Style::default().with_font_size(18).bold();
                    let sectiontitle = splitline[1];
                    doc.push(
                        Paragraph::default().styled_string(format!("{sectiontitle}: "), parstyle),
                    )
                }

                "#+BREAK" | "#+" => {
                    let sizeresult = splitline[1].parse::<f64>();
                    if let Ok(size) = sizeresult {
                        doc.push(Break::new(size));
                    } else {
                        doc.push(Break::new(1.0));
                    }
                }
                _ => {}
            }
        }
    }

    // Write output file
    doc.render_to_file("output.pdf")
        .expect("failed to write output file");
}

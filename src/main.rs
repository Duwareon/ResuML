use genpdf::{elements::*, style::*, *};
use std::{env, fs};

// Default visual values
const TITLESIZE: u8 = 28;
const SUBTITLESIZE: u8 = 18;
const ITEMSIZE: u8 = 16;
const SECTIONSIZE: u8 = 18;
const DEFAULTSIZE: u8 = 15;
const MARGINS: u8 = 20;
const FONTPATH: &str = "/usr/share/fonts/vollkorn";
const FONTNAME: &str = "Vollkorn";

const BACKGROUND: Color = Color::Rgb(0xff, 0xff, 0xff);
const FOREGROUND: Color = Color::Rgb(0x00, 0x00, 0x00);

const SPACER: &str = "\t";

fn getpar(text: &str, parstyle: Style, alignment: Alignment) -> Paragraph {
    Paragraph::default()
        .styled_string(text, parstyle)
        .aligned(alignment)
}

fn getparindent(text: &str, parstyle: Style, indent: usize) -> Paragraph {
    let invis = parstyle.with_color(BACKGROUND);
    let spacerstring = SPACER.repeat(indent);
    getpar(spacerstring.as_str(), invis, Alignment::Left).styled_string(text, parstyle)
}

pub fn main() {
    // Parse command line arguments and flags.
    let args: Vec<String> = env::args().filter(|x| !x.starts_with('-')).collect();
    if args.len() <= 1 {
        panic!("Please input a ResuMarkup file")
    }
    let flags: Vec<String> = env::args().filter(|x| x.starts_with('-')).collect();

    // Get the input .rm file to work on.
    let sourcefile = fs::read_to_string(args[1].clone()).expect("Cannot open file");

    // Set up stack for loop.
    let mut textstack: Vec<&str> = sourcefile.split('\n').collect();
    textstack.reverse();

    let firstline = textstack.pop();

    // Set visual options to default.
    let mut titlesize: u8 = TITLESIZE;
    let mut subtitlesize: u8 = SUBTITLESIZE;
    let mut itemsize: u8 = ITEMSIZE;
    let mut sectionsize: u8 = SECTIONSIZE;
    let mut defaultsize: u8 = DEFAULTSIZE;
    let mut margins: u8 = MARGINS;
    let mut fontpath: &str = FONTPATH;
    let mut fontname: &str = FONTNAME;

    // Change visuals to what is specified in #+SETUP
    // TODO: Give each setting its own command
    if let Some(line) = firstline {
        if line.starts_with("#+SETUP:") {
            let values: Vec<&str> = line.split(':').collect();
            titlesize = values[1].parse::<u8>().expect("Invalid setup parameter(s)");
            subtitlesize = values[2].parse::<u8>().expect("Invalid setup parameter(s)");
            itemsize = values[3].parse::<u8>().expect("Invalid setup parameter(s)");
            sectionsize = values[4].parse::<u8>().expect("Invalid setup parameter(s)");
            defaultsize = values[5].parse::<u8>().expect("Invalid setup parameter(s)");
            margins = values[6].parse::<u8>().expect("Invalid setup parameter(s)");
            fontpath = values[7];
            fontname = values[8];
        }
    }

    // Parse flags.
    for i in flags {
        println!("Option not recognized: {i}")
    }

    // Declare variables for pdf.
    let font = fonts::from_files(fontpath, fontname, None).expect("Failed to load font");

    let mut doc = Document::new(font);

    // Set PDF options.
    let mut decorator = SimplePageDecorator::new();
    decorator.set_margins(margins);
    doc.set_page_decorator(decorator);
    doc.set_font_size(defaultsize);
    doc.set_title("Resume");

    // Iterate through source file and add elements to PDF.
    while !textstack.is_empty() {
        let line = textstack.pop().unwrap();

        if line.starts_with("#+") {
            let splitline: Vec<&str> = line.split(':').map(|s| s.trim()).collect();
            let linestart = splitline[0];

            match linestart {
                // Write applicant name as title.
                "#+AUTHOR" => {
                    let parstyle: Style = Style::default().with_font_size(titlesize).bold();
                    doc.push(getpar(splitline[1], parstyle, Alignment::Center));
                }

                "#+INFO" => {
                    let parstyle: Style = Style::default();
                    let outputtext = splitline[1]
                        .chars()
                        .map(|c| if c == '^' { ':' } else { c })
                        .collect::<String>();

                    doc.push(getpar(outputtext.as_str(), parstyle, Alignment::Center))
                }

                // Centered subtitle for under applicant name.
                "#+SUBTITLE" => {
                    let parstyle: Style = Style::default().with_font_size(subtitlesize).italic();
                    doc.push(getpar(splitline[1], parstyle, Alignment::Center))
                }

                // Write text with bullet point.
                "#+POINT" => {
                    doc.push(BulletPoint::new(Paragraph::new(splitline[1])).with_bullet("-"));
                }

                // Write section title for an experience entry.
                "#+EXPERIENCE" => {
                    let parstyle: Style = Style::default().with_font_size(itemsize);
                    doc.push(getpar(splitline[1], parstyle, Alignment::Left));
                }

                // Write section subtitle for experience entry.
                // TODO: Figure out a better word than specialization.
                "#+SPECIALIZATION" => {
                    let parstyle: Style = Style::default().italic();
                    doc.push(getparindent(splitline[1], parstyle, 2));
                }

                // Write start and end date.
                "#+START" => {
                    let parstyle: Style = Style::default();

                    if !textstack.is_empty() {
                        let start = splitline[1];
                        let nextline = textstack.pop().unwrap();
                        if nextline.starts_with("#+END:") {
                            let endsplit: Vec<&str> =
                                nextline.split(':').map(|s| s.trim()).collect();
                            let end = endsplit[1];
                            doc.push(getparindent(
                                format!("{start} - {end}").as_str(),
                                parstyle,
                                2,
                            ));
                        } else {
                            textstack.push(nextline);
                            doc.push(getparindent(
                                format!("{start} - Present").as_str(),
                                parstyle,
                                2,
                            ))
                        }
                    }
                }

                "#+END" => {
                    let parstyle: Style = Style::default();
                    doc.push(getparindent(splitline[1], parstyle, 2))
                }

                // Set section of resume (e.g. Education, Experience, Skills).
                "#+STARTSECTION" => {
                    let parstyle: Style = Style::default().with_font_size(sectionsize).bold();
                    let sectiontitle = splitline[1];
                    doc.push(getpar(
                        format!("{sectiontitle}").as_str(),
                        parstyle,
                        Alignment::Left,
                    ))
                }

                // Basically just draw a horizontal line as a seperator.
                // Unfortunately the underlying PDF shape settings are not easy to access,
                // meaning the string of dashes will have to do for now.
                "#+ENDSECTION" => {
                    let parstyle: Style = Style::default().with_font_size(titlesize).bold();
                    let linelength = splitline[1]
                        .parse::<usize>()
                        .expect("ERROR: ENDSECTION with incorrect input");
                    let linestring = "-".repeat(linelength);
                    doc.push(getpar(linestring.as_str(), parstyle, Alignment::Center));
                }

                // Add a simple linebreak.
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

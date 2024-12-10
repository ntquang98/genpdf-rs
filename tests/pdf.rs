// SPDX-FileCopyrightText: 2021 Robin Krahl <robin.krahl@ireas.org>
// SPDX-License-Identifier: CC0-1.0

use genpdf::{elements, fonts, style, Alignment, Element as _};

const FONT_DIRS: &[&str] = &[
    "/usr/share/fonts/liberation",
    "/usr/share/fonts/truetype/liberation",
];
const DEFAULT_FONT_NAME: &'static str = "LiberationSans";

const LOREM_IPSUM: &'static str =
    "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut \
    labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco \
    laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in \
    voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat \
    non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

/// Creates a new document with the default font, minimal conformance and constant creation and
/// modification dates.
fn get_document() -> genpdf::Document {
    let font_dir = FONT_DIRS
        .iter()
        .filter(|path| std::path::Path::new(path).exists())
        .next()
        .expect("Could not find font directory");
    let default_font =
        fonts::from_files(font_dir, DEFAULT_FONT_NAME, Some(fonts::Builtin::Helvetica))
            .expect("Failed to load the default font family");

    let mut doc = genpdf::Document::new(default_font);
    doc.set_minimal_conformance();
    doc.set_creation_date(printpdf::OffsetDateTime::unix_epoch());
    doc.set_modification_date(printpdf::OffsetDateTime::unix_epoch());
    doc
}

/// Compares the PDF file generated by the given document with the stored PDF file at
/// `tests/files/<name>.pdf`.
fn check(name: &str, doc: genpdf::Document) {
    let expected_dir = std::path::Path::new("tests/files");
    if !expected_dir.exists() {
        std::fs::create_dir(&expected_dir).expect("Failed to create expected directory");
    }

    let expected_path = expected_dir.join(name).with_extension("pdf");
    if expected_path.exists() {
        std::fs::remove_file(&expected_path).expect("Failed to delete existing file");
    }

    let mut actual_doc: Vec<u8> = Vec::new();
    doc.render(&mut actual_doc)
        .expect("Failed to render document");

    // Prune ID because it is randomly generated by printpdf.
    let mut actual_pdf_doc =
        lopdf::Document::load_mem(&actual_doc).expect("Failed to load actual document");
    actual_pdf_doc.trailer.remove(b"ID");
    let mut actual_doc: Vec<u8> = Vec::new();
    actual_pdf_doc
        .save_to(&mut actual_doc)
        .expect("Failed to save pruned actual document");

    let expected_path = expected_dir.join(name).with_extension("pdf");
    if expected_path.exists() {
        let expected_doc = std::fs::read(&expected_path).expect("Failed to read expected document");
        if actual_doc != expected_doc {
            let actual_path = expected_path.with_extension("pdf.new");
            std::fs::write(&actual_path, actual_doc).expect("Failed to write actual document");
            panic!(
                "Actual document does not match expected document.  Please check {} \
                 for more information",
                actual_path.display(),
            );
        }
    } else {
        std::fs::write(expected_path, actual_doc).expect("Failed to write expected document");
    }
}

macro_rules! test_with_document {
    ($( $(#[$outer:meta])* fn $name:ident($arg:ident: $arg_ty:ty) -> $ret_ty:ty $body:block )*) => {
        $(
            $(#[$outer])*
            fn $name() {
                let $arg: $arg_ty = get_document();
                let doc: $ret_ty = $body;
                check(stringify!($name), doc);
            }
        )*
    };
}

test_with_document! {
    #[test]
    fn minimal(doc: genpdf::Document) -> genpdf::Document {
        doc
    }

    #[test]
    fn text(doc: genpdf::Document) -> genpdf::Document {
        // TODO: Why 14pt and not 12?
        let mut doc = doc;
        doc.set_paper_size((12, genpdf::Mm::from(printpdf::Pt(14.0))));
        doc.push(elements::Text::new("foobar"));
        doc
    }

    #[test]
    // Ignore as this currently returns an error
    #[ignore]
    fn paragraph_long(doc: genpdf::Document) -> genpdf::Document {
        let mut doc = doc;
        doc.set_paper_size((50, 100));
        doc.push(elements::Paragraph::new("Donaudampfschifffahrtskapitänsmützenhersteller"));
        doc
    }

    #[test]
    fn kerning(doc: genpdf::Document) -> genpdf::Document {
        let mut doc = doc;
        doc.set_paper_size((7, 10));
        doc.push(elements::Paragraph::new("AV"));
        doc.push(elements::Paragraph::new("A").string("V"));
        doc
    }

    #[test]
    #[ignore]
    fn sizes(doc: genpdf::Document) -> genpdf::Document {
        // TODO: Top/bottom spacing
        let mut doc = doc;
        doc.set_paper_size((25, 50));
        for size in &[5, 10, 20, 40, 5, 20, 10] {
            doc.push(
                elements::Text::new("zyp")
                    .styled(style::Style::new().with_font_size(*size))
            );
        }
        doc
    }

    #[test]
    fn frame_single(doc: genpdf::Document) -> genpdf::Document {
        let mut doc = doc;
        doc.set_paper_size((100, 30));

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(5);
        doc.set_page_decorator(decorator);

        doc.push(
            elements::Paragraph::new("Lorem ipsum")
                .framed(style::LineStyle::new())
        );

        doc
    }

    #[test]
    fn frame_multi(doc: genpdf::Document) -> genpdf::Document {
        let mut doc = doc;
        doc.set_paper_size((100, 30));

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(5);
        doc.set_page_decorator(decorator);

        doc.push(
            elements::Paragraph::new(LOREM_IPSUM)
                .framed(style::LineStyle::new())
        );

        doc
    }

    #[test]
    fn frame_single_thick(doc: genpdf::Document) -> genpdf::Document {
        let mut doc = doc;
        doc.set_paper_size((100, 30));

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(5);
        doc.set_page_decorator(decorator);

        doc.push(
            elements::Paragraph::new("Lorem ipsum")
                .framed(style::LineStyle::new().with_thickness(5))
        );

        doc
    }

    #[test]
    fn frame_multi_thick(doc: genpdf::Document) -> genpdf::Document {
        let mut doc = doc;
        doc.set_paper_size((100, 30));

        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(5);
        doc.set_page_decorator(decorator);

        doc.push(
            elements::Paragraph::new(LOREM_IPSUM)
                .framed(style::LineStyle::new().with_thickness(5))
        );

        doc
    }

    #[test]
    // Ignore as this currently returns an error
    // #[ignore]
    fn table(doc: genpdf::Document) -> genpdf::Document {
        let mut doc = doc;
        let mut decorator = genpdf::SimplePageDecorator::new();
        decorator.set_margins(10);
        decorator.set_header(|page| {
            let mut layout = elements::LinearLayout::vertical();
            if page > 1 {
                layout.push(
                    elements::Paragraph::new(format!("Page {}", page)).aligned(Alignment::Center),
                );
                layout.push(elements::Break::new(1));
            }
            layout.styled(style::Style::new().with_font_size(10))
        });
        doc.set_page_decorator(decorator);
        let mut table = elements::TableLayout::new(vec![2, 2]);
        table.set_cell_decorator(elements::FrameCellDecorator::new(true, true, true));
        table
            .row()
            .set_background_color(style::Color::Rgb(59, 59, 59))
            .element(elements::Paragraph::new(
                "Vendor: Công ty TNHH Gigamed",
            )
            .styled(style::Style::new().with_color(style::Color::Rgb(255, 255, 255)))
        )
            .element(
                elements::Paragraph::new("PO658597").aligned(Alignment::Right)
                .styled(style::Style::new().with_color(style::Color::Rgb(255, 255, 255))),
            )
            .push()
            .expect("invalid table row");

        table
            .row()
            .element(elements::Paragraph::new(""))
            .element(elements::Paragraph::new("PO658597").aligned(Alignment::Right))
            .push()
            .expect("invalid table row");

        doc.push(table);
        doc.push(elements::Paragraph::new("Donaudampfschifffahrtskapitänsmützenhersteller"));
        doc
    }
}

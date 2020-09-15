mod glyph;

use inline_python::python;
use std::process::Command;
use std::str;
use sxd_document::{dom, parser, writer};
use sxd_xpath::{evaluate_xpath, Value};

const OTFFILE: &str = "./vianaic.final.otf";
const TTXFILE: &str = "./target/vianaic.ttx";
const OTFSRCFILE: &str = "./vianaic.src.otf";

fn find_elt<'a>(doc: &'a dom::Document, query: &str) -> Option<dom::Element<'a>> {
    match evaluate_xpath(&doc, query).expect("XPath evaluation failed") {
        Value::Nodeset(value) => value.document_order_first().map(|e| e.element().unwrap()),
        _ => panic!("XXX"),
    }
}

fn main() {
    // FIXME: check that ttx exists
    let out = Command::new("ttx")
        .arg("-o")
        .arg("-")
        .arg(&OTFSRCFILE)
        .output()
        .expect("failed to execute ttx");
    if !out.status.success() {
        panic!("XXX");
    }

    let xml = str::from_utf8(&out.stdout).unwrap();
    let pkg = parser::parse(xml).expect("Failed to parse");
    let doc = pkg.as_document();

    let e = find_elt(&doc, "//cmap").unwrap();
    let e1 = find_elt(
        &doc,
        "//cmap/cmap_format_4[@platformID=\"0\"][@platEncID=\"3\"]",
    )
    .unwrap();
    let e2 = find_elt(&doc, "//cmap/tableVersion").unwrap();
    e.clear_children();
    e.append_child(e1);
    e.append_child(e2);

    let all = glyph::create_all();

    all.iter().for_each(|g| {
        let id = g.source;
        let new_name = g.name;
        let elt = find_elt(&doc, &format!("//map[@name=\"{}\"]", id)).unwrap();
        elt.set_attribute_value("name", new_name);
        let elt = find_elt(&doc, &format!("//GlyphID[@name=\"{}\"]", id)).unwrap();
        elt.set_attribute_value("name", new_name);
        let elt = find_elt(&doc, &format!("//CharString[@name=\"{}\"]", id)).unwrap();
        elt.set_attribute_value("name", new_name);
        let elt = find_elt(&doc, &format!("//mtx[@name=\"{}\"]", id)).unwrap();
        elt.set_attribute_value("name", new_name);
        let elt = find_elt(&doc, &format!("//ClassDef[@glyph=\"{}\"]", id)).unwrap();
        elt.set_attribute_value("glyph", new_name);
    });

    glyph::CODES.iter().for_each(|code| {
        let code_str = format!("{:#x}", code.0 as u8);
        let query = format!("//map[@code=\"{}\"]", code_str);
        let elt = match find_elt(&doc, &query) {
            Some(elt) => elt,
            None => {
                let elt = doc.create_element("map");
                elt.set_attribute_value("code", &code_str);
                find_elt(&doc, "//cmap_format_4").unwrap().append_child(elt);
                elt
            }
        };
        elt.set_attribute_value("name", code.1);
    });

    // DEBUG:
    // writer::format_document(&doc, &mut std::io::stdout()).expect("unable to output XML");

    let mut f = std::fs::File::create(&TTXFILE).unwrap();

    writer::format_document(&doc, &mut f).expect("unable to output XML");

    let out = Command::new("ttx")
        .arg("-o")
        .arg(&OTFFILE)
        .arg(&TTXFILE)
        .output()
        .expect("failed to execute ttx");
    if !out.status.success() {
        panic!("XXX")
    }

    let class_any: Vec<&'static str> = all.iter().map(|g| g.name).collect();
    let class_right: Vec<&'static str> = all.iter().filter(|g| g.right).map(|g| g.name).collect();
    let class_left: Vec<&'static str> = all.iter().filter(|g| g.left).map(|g| g.name).collect();
    let class_swappable_right: Vec<&'static str> = all
        .iter()
        .filter(|g| g.right && g.swappable)
        .map(|g| g.name)
        .collect();
    let class_swappable_left: Vec<&'static str> = all
        .iter()
        .filter(|g| g.left && g.swappable)
        .map(|g| g.name)
        .collect();

    let rules = format!(
        r#"
	languagesystem DFLT dflt;
	languagesystem latn dflt;

	@ANY=[{class_any}];
	@RIGHT=[{class_right}];
	@LEFT=[{class_left}];
	@SWAPPABLE_RIGHT=[{class_swappable_right}];
	@SWAPPABLE_LEFT=[{class_swappable_left}];

	feature liga {{
	    sub _c_ h_ by ch_;
	    sub t_ h_ by th_;
	    sub s_ h_ by sh_;
	    sub g_ h_ by gh_;
	    sub _n g_ by _ng;
	    sub w_ h_ by wh_;
	    sub p_ h_ by ph_;

	    # Doesn't work so well in a word

	    ignore sub @ANY a' _n' _d_';
	    ignore sub a' _n' _d_' @ANY;
	    sub a' _n' _d_' by and;

	    ignore sub @ANY t_' h_' e_';
	    ignore sub t_' h_' e_' @ANY;
	    sub t_' h_' e_' by the;
	}} liga;

	feature kern {{
	    position @RIGHT' @LEFT -100;
        }} kern;

	feature calt {{
	    sub @RIGHT @SWAPPABLE_RIGHT' by @SWAPPABLE_LEFT;
	}} calt;
    "#,
        class_any = class_any.join(" "),
        class_right = class_right.join(" "),
        class_left = class_left.join(" "),
        class_swappable_right = class_swappable_right.join(" "),
        class_swappable_left = class_swappable_left.join(" "),
    );

    // DEBUG:
    // println!("{}", rules);

    python! {
        import sys, getopt
        from fontTools.ttLib import TTFont
        from fontTools.feaLib.builder import addOpenTypeFeaturesFromString
        font = TTFont('OTFFILE)
        addOpenTypeFeaturesFromString(font, 'rules)
        font.save('OTFFILE)
    }
}
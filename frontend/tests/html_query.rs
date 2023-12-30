use frontend::app::SeedMsg;
use seed::virtual_dom::{At, El, Node};
use std::fmt::Display;

use syntect::easy::HighlightLines;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use syntect::util::{as_24_bit_terminal_escaped, LinesWithEndings};

/// Check that an html node tree contains at least one node which text contains `text`
pub fn assert_contains_text(html: &Node<SeedMsg>, text: &str) {
    assert!(
        matches!(get_element_by_contents(html, text), Some(..)),
        "the view does not contain a node with the text 'event 1 name':\n{}",
        highlight_html_syntax(&indent(&html))
    )
}

/// Get the first element of type seed::virtual_dom::El containing a Text node which text contains `contents`
fn get_element_by_contents<'a>(node: &'a Node<SeedMsg>, contents: &str) -> Option<&'a El<SeedMsg>> {
    // search children only if the current node is an element
    if let Node::Element(current_el) = node {
        // the current element has a text child containing `contents` -> return it
        if current_el
            .children
            .iter()
            .filter(|child| child.is_text() && child.text().unwrap().text.contains(contents))
            .count()
            > 0
        {
            Some(current_el)
        }
        // check if any child element has a text child containing `contents`
        else {
            current_el
                .children
                .iter()
                .filter(|child| child.is_el())
                .map(|child| get_element_by_contents(child, contents))
                .find(|child| child.is_some())?
        }
    } else {
        None
    }
}

fn indent(node: &Node<SeedMsg>) -> String {
    IndentedHtml { node: node }.to_string()
}

fn highlight_html_syntax(html: &str) -> String {
    let ps = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let syntax = ps.find_syntax_by_extension("html").unwrap();
    let mut h = HighlightLines::new(syntax, &ts.themes["base16-ocean.dark"]);
    LinesWithEndings::from(html)
        .map(|line| h.highlight_line(line, &ps).unwrap())
        .map(|ranges| as_24_bit_terminal_escaped(&ranges[..], true))
        .collect::<Vec<String>>()
        .concat()
        + "\x1b[0m" /* clear */
}

struct IndentedHtml<'a> {
    node: &'a Node<SeedMsg>,
}

impl<'a> IndentedHtml<'a> {
    fn write_node(
        node: &'a Node<SeedMsg>,
        f: &mut std::fmt::Formatter<'_>,
        indentation: usize,
    ) -> std::fmt::Result {
        match node {
            Node::Element(el) => {
                let tag = el.tag.to_string();
                let mut open_tag = format!("<{}", &tag);
                let mut attrs = el.attrs.clone();
                let style = el.style.to_string();
                if !style.is_empty() {
                    attrs.add(At::Style, style);
                }
                if let Some(namespace) = el.namespace.as_ref() {
                    attrs.add(At::Xmlns, namespace.as_str());
                }
                let attributes = attrs.to_string();
                if !attributes.is_empty() {
                    open_tag += &format!(" {}", attributes);
                }
                open_tag += ">";
                if el.children.len() > 1 {
                    open_tag += "\n";
                }
                write!(f, "{}{}", "  ".repeat(indentation), open_tag)?;

                if el.children.len() > 1 {
                    for child in &el.children {
                        IndentedHtml::write_node(child, f, indentation + 1)?;
                        write!(f, "\n")?;
                    }
                    write!(f, "{}</{}>", "  ".repeat(indentation), tag)?;
                } else {
                    for child in &el.children {
                        IndentedHtml::write_node(child, f, 0)?;
                    }
                    write!(f, "</{}>", tag)?;
                }

                Ok(())
            }
            Node::Text(text) => write!(f, "{}{}", "  ".repeat(indentation), text),
            Node::Empty => write!(f, ""),
            Node::NoChange => write!(f, ""),
        }
    }
}

impl<'a> Display for IndentedHtml<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        IndentedHtml::write_node(self.node, f, 0)
    }
}

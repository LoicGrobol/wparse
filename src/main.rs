use lazy_static::lazy_static;
use rayon::prelude::*;
use std::borrow::Cow;

mod frconfig;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: wparse <dump_path> <out_path>");
        std::process::exit(1);
    }
    let dump_path = &args[1];
    let dump_file = match std::fs::File::open(&dump_path) {
        Err(error) => {
            eprintln!("Failed to open dump file: {}", error);
            std::process::exit(1);
        }
        Ok(file) => std::io::BufReader::new(file),
    };
    let out_path = &args[2];
    let out_file = match std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .open(out_path)
    {
        Err(error) => {
            eprintln!("Failed to open output file: {}", error);
            std::process::exit(1);
        }
        Ok(file) => std::io::BufWriter::new(file),
    };
    if dump_path.ends_with(".bz2") {
        parse_dump(
            std::io::BufReader::new(bzip2::bufread::BzDecoder::new(dump_file)),
            out_file,
        );
    } else {
        parse_dump(dump_file, out_file);
    }
}

fn parse_dump(source: impl std::io::BufRead, mut target: impl std::io::Write) {
    let config = frconfig::create_configuration();
    // TODO: see if that might be parallelized
    for result in parse_mediawiki_dump::parse(source) {
        match result {
            Err(error) => {
                eprintln!("Error: {}", error);
            }
            Ok(page) => {
                if [0, 1, 4, 12].contains(&page.namespace)  // Main, Talk, Wikipédia, Help
                    && match &page.format {
                        None => false,
                        Some(format) => format == "text/x-wiki",
                    }
                    && match &page.model {
                        None => false,
                        Some(model) => model == "wikitext",
                    }
                {
                    match parse_page(page, &config) {
                        Err(error) => {
                            eprintln!("Error: {}", error);
                        }
                        Ok(Some(raw)) => {
                            write!(target, "{}", raw).unwrap();
                        }
                        _ => ()
                    }
                } else {
                    eprintln!(
                        "Skipping page {:?}: it has namespace {:?}",
                        page.title, page.namespace
                    );
                }
            }
        }
        // let mut inpt = String::new();
        // std::io::stdin().read_line(&mut inpt).unwrap();
    }
}

fn parse_page(
    page: parse_mediawiki_dump::Page,
    config: &parse_wiki_text::Configuration,
) -> Result<Option<String>, &'static str> {
    eprintln!("=== {} ===", page.title);
    let mut text = page.text;
    text = text.replace("{{fin}}", "|}"); // This is the proof of wikitext evil
    let result = config.parse(&text);
    eprintln!("Parsed!");
    for w in result.warnings {
        eprintln!("{}", w.message)
    }
    if let Some(parse_wiki_text::Node::Redirect{ .. }) = result.nodes.first(){
        return Ok(None)
    }
    // assert!(result.warnings.is_empty());
    return Ok(Some(dedup(&parse_nodes(&result.nodes))));
}


fn parse_nodes(nodes: &[parse_wiki_text::Node]) -> String {
    nodes.par_iter().filter_map(parse_node).collect::<String>()
}


fn parse_node<'a>(node: &'a parse_wiki_text::Node) -> Option<std::borrow::Cow<'a, str>> {
     match node {
        parse_wiki_text::Node::Text { value, .. } => Some(Cow::Borrowed(value)),
        parse_wiki_text::Node::ParagraphBreak { .. } => Some(Cow::Borrowed("\n")),
        parse_wiki_text::Node::Link { text, .. } => Some(Cow::Owned(parse_nodes(text))),
            // FIXME: Other links such as `[https://example.com <nowiki>a</nowiki>]` are ignored for now
        parse_wiki_text::Node::ExternalLink { nodes, .. } if !nodes.is_empty() =>
            match nodes.split_first() {
                Some((parse_wiki_text::Node::Text { value, .. }, rest)) => {
                    let mut outpt = String::new();
                    // The first node is of the form `http://example.com some text`
                    // where the text might be empty
                    match value.splitn(2, ' ').collect::<Vec<&str>>().split_first() {
                        Some((_, text)) if !text.is_empty() => outpt.push_str(text[0]),
                        _ => (),
                    }
                    outpt.push_str(parse_nodes(rest).as_str());
                    return Some(Cow::Owned(outpt));
                }
                _ => None,
            }
        parse_wiki_text::Node::Image { text, .. } => {
            let mut outpt = String::new();
            if let Some((parse_wiki_text::Node::Text { value, .. }, rest)) = text.split_first() {
                outpt.push_str(value.rsplitn(2, '|').next().unwrap());
                if !rest.is_empty(){
                    outpt.push_str(parse_nodes(rest).as_str());
                }
            } else {
                outpt.push_str(parse_nodes(text).as_str());
            }
            outpt.push_str("\n");
            return Some(Cow::Owned(outpt));
        }
        parse_wiki_text::Node::CharacterEntity { character, .. } => Some(Cow::Owned(character.to_string())),
        parse_wiki_text::Node::UnorderedList { items, .. }
        | parse_wiki_text::Node::OrderedList { items, .. } => {
            let mut outpt = String::new();
            outpt.push_str("\n");
            outpt.push_str(
                items
                    .par_iter()
                    .map(|item| parse_nodes(&item.nodes))
                    .collect::<Vec<String>>()
                    .join("\n")
                    .as_str(),
            );
            outpt.push_str("\n");
            return Some(Cow::Owned(outpt));
        }
        parse_wiki_text::Node::DefinitionList { items, .. } => {
            let mut outpt = String::new();
            outpt.push_str("\n");
            outpt.push_str(
                items
                    .par_iter()
                    .map(|item| parse_nodes(&item.nodes))
                    .collect::<Vec<String>>()
                    .join("\n")
                    .as_str(),
            );
            outpt.push_str("\n");
            return Some(Cow::Owned(outpt));
        }
        // parse_wiki_text::Node::Heading { nodes, .. } => {
        //     let mut outpt = String::new();
        //     outpt.push_str("\n");
        //     outpt.push_str(parse_nodes(&nodes).as_str());
        //     outpt.push_str("\n");
        //     return Some(Cow::Owned(outpt))
        // }
        parse_wiki_text::Node::Template { .. } => Some(Cow::Owned(parse_template(node).unwrap())),
        _ => None,
    }
}


fn parse_template(node: &parse_wiki_text::Node) -> Result<String, &'static str> {
    match node {
        parse_wiki_text::Node::Template {
            name, parameters, ..
        } => {
            let mut outpt = String::new();
            // TODO: extract templates with constant expansions
            if !name.is_empty() {
                if let parse_wiki_text::Node::Text { value, .. } = name[0] {
                    match value.to_lowercase().as_str() {
                        "date" | "date-" | "date de naissance" | "date de décès" | "unité"
                        | "nombre" | "nobr" => outpt.push_str(
                            parameters
                                .iter()
                                .map(|p| parse_nodes(&p.value))
                                .collect::<Vec<String>>()
                                .join(" ")
                                .as_str(),
                        ),
                        "lien" => {
                            // This is used for interwikis instead of [[ ]]
                            let named_params: std::collections::HashMap<String, String> =
                                parameters
                                    .iter()
                                    .filter_map(|p| match &p.name {
                                        Some(name) => {
                                            Some((parse_nodes(name), parse_nodes(&p.value)))
                                        }
                                        _ => None,
                                    })
                                    .collect();
                            if let Some(text) = named_params.get("texte") {
                                outpt.push_str(text.as_str());
                            } else {
                                match named_params.get("fr") {
                                    Some(text) => outpt.push_str(text.as_str()),
                                    _ => {
                                        if let Some(parse_wiki_text::Parameter { value, .. }) =
                                            parameters.iter().filter(|p| !p.name.is_some()).next()
                                        {
                                            outpt.push_str(parse_nodes(&value).as_str())
                                        }
                                    }
                                }
                            }
                        }
                        "s-" | "sav-" => {
                            if !parameters.is_empty() {
                                outpt.push_str(&parse_nodes(&parameters[0].value));
                                outpt.push_str("e siècle");
                            }
                        }
                        "article" | "chapitre" | "ouvrage" => {
                            for p in parameters {
                                if let Some(name) = &p.name {
                                    if !name.is_empty() {
                                        if let parse_wiki_text::Node::Text { value, .. } = name[0] {
                                            if value == "titre" {
                                                outpt.push_str(&parse_nodes(&p.value))
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        _ => {
                            if value.contains("siècle") || value.contains("millénaire") {
                                outpt.push_str(value);
                            } else if value.starts_with("formatnum:") {
                                outpt.push_str(&value[10..]);
                            }
                        }
                    }
                }
            }
            return Ok(outpt);
        }
        _ => Err("Not a template node"),
    }
}

fn dedup(s: &String) -> String {
    lazy_static! {
       static ref re: regex::Regex = regex::Regex::new(r"\n\s*").unwrap();
    }
    return String::from(re.replace_all(s, "\n"));
}

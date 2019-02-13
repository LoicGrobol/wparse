use bzip2;
use parse_mediawiki_dump;
use parse_wiki_text;
use regex;

mod frconfig;
// use regex;

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
                        Ok(raw) => {
                            write!(target, "{}", raw).unwrap();
                        }
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

// TODO: skip redirects
fn parse_page(
    page: parse_mediawiki_dump::Page,
    config: &parse_wiki_text::Configuration,
) -> Result<String, &'static str> {
    eprintln!("=== {} ===", page.title);
    let mut text = page.text;
    text = text.replace("{{fin}}", "|}"); // This is the proof of wikitext evil
    let result = config.parse(&text);
    eprintln!("Parsed!");
    for w in result.warnings {
        eprintln!("{}", w.message)
    }
    // assert!(result.warnings.is_empty());
    return Ok(dedup(&parse_nodes(&result.nodes)));
}

fn parse_nodes(nodes: &[parse_wiki_text::Node]) -> String {
    let mut outpt = String::new();
    for node in nodes {
        match node {
            parse_wiki_text::Node::Text { value, .. } => outpt.push_str(value),
            parse_wiki_text::Node::ParagraphBreak { .. } => outpt.push_str("\n"),
            parse_wiki_text::Node::Link { text, .. } => outpt.push_str(parse_nodes(text).as_str()),
            parse_wiki_text::Node::ExternalLink { nodes, .. } if !nodes.is_empty() => {
                if let Some((parse_wiki_text::Node::Text { value, .. }, rest)) = nodes.split_first()
                {
                    // The first node is of the form `http://example.com some text`
                    // where the text might be empty
                    match value.splitn(2, ' ').collect::<Vec<&str>>().split_first() {
                        Some((_, text)) if !text.is_empty() => outpt.push_str(text[0]),
                        _ => (),
                    }
                    outpt.push_str(parse_nodes(rest).as_str());
                } else {
                    panic!("Weird invalid extlink: {:?}", node)
                }
            }
            parse_wiki_text::Node::Image { text, .. } => {
                if let Some((parse_wiki_text::Node::Text { value, .. }, rest)) = text.split_first() {
                    outpt.push_str(value.rsplitn(2, '|').next().unwrap());
                    if !rest.is_empty(){
                        outpt.push_str(parse_nodes(rest).as_str());
                    }
                } else {
                    outpt.push_str(parse_nodes(text).as_str());
                }
            }
            parse_wiki_text::Node::CharacterEntity { character, .. } => outpt.push(*character),
            parse_wiki_text::Node::UnorderedList { items, .. }
            | parse_wiki_text::Node::OrderedList { items, .. } => {
                outpt.push_str(
                    items
                        .iter()
                        .map(|item| parse_nodes(&item.nodes))
                        .collect::<Vec<String>>()
                        .join(" ")
                        .as_str(),
                );
                outpt.push_str("\n");
            }
            parse_wiki_text::Node::DefinitionList { items, .. } => {
                outpt.push_str(
                    items
                        .iter()
                        .map(|item| parse_nodes(&item.nodes))
                        .collect::<Vec<String>>()
                        .join(" ")
                        .as_str(),
                );
                outpt.push_str("\n");
            }
            parse_wiki_text::Node::Heading { nodes, .. } => {
                outpt.push_str("\n");
                outpt.push_str(parse_nodes(&nodes).as_str());
                outpt.push_str("\n");
            }
            parse_wiki_text::Node::Template { .. } => {
                outpt.push_str(parse_template(node).unwrap().as_str())
            }
            _ => (),
        }
    }
    return outpt;
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
                            } else {
                                eprintln!("Discard template: {}", value)
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
    let re = regex::Regex::new(r"\n\s*").unwrap();
    return String::from(re.replace_all(s, "\n"));
}

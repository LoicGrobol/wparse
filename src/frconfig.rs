pub fn create_configuration() -> ::parse_wiki_text::Configuration {
    ::parse_wiki_text::Configuration::new(&::parse_wiki_text::ConfigurationSource {
        category_namespaces: &[
            "category",
            "catégorie",
        ],
        extension_tags: &[
            "categorytree",
            "ce",
            "charinsert",
            "chem",
            "gallery",
            "graph",
            "hiero",
            "imagemap",
            "indicator",
            "inputbox",
            "mapframe",
            "maplink",
            "math",
            "nowiki",
            "poem",
            "pre",
            "ref",
            "references",
            "score",
            "section",
            "source",
            "syntaxhighlight",
            "templatedata",
            "templatestyles",
            "timeline",
        ],
        file_namespaces: &[
            "fichier",
            "file",
            "image",
        ],
        link_trail: "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyzÀÂÄÇÈÉÊËÎÏÔÖÙÛÜàâäçèéêëîïôöùûü",
        magic_words: &[
            "AUCUNEGALERIE",
            "AUCUNETDM",
            "AUCUNINDEX",
            "AUCUNLIENNOUVELLESECTION",
            "AUCUNSOMMAIRE",
            "CATCACHEE",
            "DISAMBIG",
            "EXPECTUNUSEDCATEGORY",
            "FORCERSOMMAIRE",
            "FORCERTDM",
            "FORCETOC",
            "HIDDENCAT",
            "INDEX",
            "LIENNOUVELLESECTION",
            "NEWSECTIONLINK",
            "NOCC",
            "NOCOLLABORATIONHUBTOC",
            "NOCONTENTCONVERT",
            "NOEDITSECTION",
            "NOGALLERY",
            "NOGLOBAL",
            "NOINDEX",
            "NONEWSECTIONLINK",
            "NOTC",
            "NOTITLECONVERT",
            "NOTOC",
            "REDIRECTIONSTATIQUE",
            "SANSCC",
            "SANSCONVERSIONCONTENU",
            "SANSCONVERSIONTITRE",
            "SANSCT",
            "SECTIONNONEDITABLE",
            "SOMMAIRE",
            "STATICREDIRECT",
            "TDM",
            "TOC",
        ],
        protocols: &[
            "//",
            "bitcoin:",
            "ftp://",
            "ftps://",
            "geo:",
            "git://",
            "gopher://",
            "http://",
            "https://",
            "irc://",
            "ircs://",
            "magnet:",
            "mailto:",
            "mms://",
            "news:",
            "nntp://",
            "redis://",
            "sftp://",
            "sip:",
            "sips:",
            "sms:",
            "ssh://",
            "svn://",
            "tel:",
            "telnet://",
            "urn:",
            "worldwind://",
            "xmpp:",
        ],
        redirect_magic_words: &[
            "REDIRECT",
            "REDIRECTION",
        ]
    })
}
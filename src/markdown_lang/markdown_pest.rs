use std::path::Path;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::common::{ParsingError, debug::*, filesystem::read_file_content};

use super::markdown_pest_error::MarkdownPestError;

#[derive(Parser)]
#[grammar = "markdown_lang/markdown.pest"]
pub (crate) struct MarkdownPestParser;

pub fn parse(input:&str) -> Result<File, ParsingError> {
    let result = MarkdownPestParser::parse(Rule::file, input);

    match result {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
                match File::from_pair(&pair) {
                    Ok(rr) => {
                        Ok(rr)
                    }
                    Err(err2) => {
                        error_string(format!("{:?}", err2));
                        Err(err2)
                    }
                }
            }
            else {
                error_string(format!("cannot match all string in {}", input));
                Err(ParsingError::MarkdownPestError(MarkdownPestError::NoMatchWholeInput))
            }
        }
        Err(err) => {
            error_string(format!("{}", err));
            Err(ParsingError::MarkdownPestError(MarkdownPestError::PasringError))
        }
    }
}

pub (self) fn from_pair_vec_template<T, T2>(pair:&Pair<Rule>, rule:Rule, f:fn(Pair<Rule>)->T2, f2:fn(Vec<T2>) -> T) -> Result<T, ParsingError> {
    if pair.as_rule() == rule {
        let inner = pair.to_owned().into_inner();
        let rules : Vec<T2> = inner.map(|x| f(x) ).collect();
        Ok(f2(rules))
    }
    else {
        println!("{:?} cannot match {:#?}", rule, pair);
        Err(ParsingError::ParsingConversionError)
    }
}

pub (self) fn from_pair_vec_null_fn_template<T2>(pair:&Pair<Rule>, rule:Rule, f:fn(Pair<Rule>)->T2) -> Result<Vec<T2>, ParsingError> {
    from_pair_vec_template(pair, rule, f, |x| { x })
}

#[derive(Debug, Clone)]
pub struct File {
    text_line : Vec<TextLine>
}

impl File {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        let v = from_pair_vec_null_fn_template(pair, Rule::file, |rule| {
            match rule.as_rule() {
                Rule::list => {
                    let l = MDList::from_pair(&rule).unwrap();
                    TextLine::BlockType(BlockType::MDList(l))
                }
                Rule::comment => {
                    TextLine::BlockType(BlockType::Comment(rule.as_str().to_string()))
                }
                Rule::codeblock => {
                    let code_block = CodeBlock::from_pair(&rule).unwrap();
                    TextLine::BlockType(BlockType::CodeBlock(code_block))
                }
                Rule::quote => {
                    let quote = Quote::from_pair(&rule).unwrap();
                    TextLine::BlockType(BlockType::Quote(quote))
                }                
                Rule::h5 | Rule::h6 |
                Rule::h2 | Rule::h3 | Rule::h4 |
                Rule::h1 => {
                    let header = Header::from_pair(&rule).unwrap();
                    TextLine::BlockType(BlockType::Header(header))
                }
                Rule::horiz_sep => TextLine::HorizontalBar,
                Rule::paragraph => {
                    let inner = rule.into_inner();
                    let mut r = Vec::default();
                    for n in inner {
                        if n.as_rule() == Rule::text || n.as_rule() == Rule::inline_symbol {
                            let item = RichTextItem::from_pair(&n).unwrap();
                            r.push(item);
                        }
                    }

                    let rich_text = RichText::from_vec(r);

                    TextLine::Paragraph(rich_text)
                }
                Rule::table => {
                    let inner = rule.into_inner();
                    let mut rows = Vec::default();
                    for row_pair in inner {
                        let mut row = Vec::default();
                        
                        for item in row_pair.into_inner() {
                            let mut cell = Vec::default();

                            let  inner = item.into_inner();
                            if inner.len() > 0 {
                                for cell_item in inner {
                                    match cell_item.as_rule() {
                                        Rule::bold | Rule::italic | Rule::inline_code | Rule::link | Rule::reflink | Rule::refurl | Rule::image | Rule::footnote_archer_text => 
                                            cell.push(RichTextItem::from_pair(&cell_item).unwrap()),
                                        Rule::text => 
                                            cell.push(RichTextItem::from_pair(&cell_item).unwrap()),
                                        _ => {
                                            error_string(format!("MD parsing missed case: parsing table cell rich text error with rules = {cell_item:?}, search for {:?} to fix", Rule::file));
                                            todo!()
                                        }
                                    }
                                }
                            }
                            else {
                                cell.push(RichTextItem::Text(String::new()))
                            }

                            let rich_text_cell = RichText::from_vec(cell);

                            row.push(rich_text_cell);
                        }

                        rows.push(row);
                    }

                    TextLine::Table(Table { rows })
                }
                Rule::footnote_content => {
                    let inner = rule.into_inner();
                    let l = inner.into_iter().map(|x| { (x.as_rule(), x) }).collect::<Vec::<_>>();
                    match &l[..] {
                        [(Rule::footnote_archer_text, p), (Rule::text, p1)] => {
                            TextLine::FootNote(p.as_str().to_string(), p1.as_str().to_string())
                        }
                        _ => {
                            error_string(format!("MD parsing missed case: parsing error with rules = {l:?}, search for {:?} to fix", Rule::footnote_content));
                            todo!() 
                        }
                    }
                }
                Rule::EOI => {
                    TextLine::END
                }
                _ => {
                    error_string(format!("MD parsing missed case: parsing error with rules = {rule:?}, search for {:?} to fix", Rule::file));
                    todo!()
                }
            }
        })?;

        Ok(Self { text_line: v })
    }

    pub fn get_tables(&self) -> Vec<&Table> {
        self.text_line.iter()
            .filter_map(|x| x.get_table())
            .collect::<Vec<_>>()
    }

    pub fn from_file<P:AsRef<Path>>(path:P) -> Result<Self, ParsingError> {
        let source = read_file_content(path).map_err(|_| ParsingError::IOError)?;
        Self::from_string(source.as_str())
    }

    pub fn from_string(str:&str) -> Result<Self, ParsingError> {
        parse(str)
    }

    pub fn get_codes(&self) -> Vec<&CodeBlock> {
        self.text_line.iter()
            .filter_map(|x| x.get_code())
            .collect::<Vec<_>>()
    }

    pub fn find_after_header<S>(&self, head_text:S, f:fn(&TextLine) -> bool) -> Option<&TextLine>
        where 
            S: AsRef<str>,  
    {
        let r = self.text_line.iter()
                    .skip_while(|x| {
                        if let Some(h) = x.get_header() {
                            let header = h.get_text();
                            if header.trim() == head_text.as_ref() {
                                false  //to stop
                            }
                            else {
                                true //to continue
                            }
                        }
                        else {
                            true
                        }
                    })
                    .find(|x| f(x) );

        r
    }

    pub fn get_all_text_lines_after_header(&self, header_text:&str, level:u8) -> Vec<&TextLine> {
        let r = self.text_line.iter()
                        .skip_while(|x| !x.is_header_with_text_and_level(header_text, level))
                        .take_while(|x| x.is_header_with_text_and_level(header_text, level) || !x.is_header_at_level(level))
                        .collect::<Vec<_>>();

        r
    }

    pub fn get_items_in_header(&self, header_text:&str, level:u8, f:fn(&TextLine) -> bool) -> Vec<&TextLine> {
        let text_lines = self.get_all_text_lines_after_header(header_text, level);
        let r = text_lines.into_iter().filter(|x| f(x)).collect::<Vec<_>>();
        r
    }

    /// get all code blocks after header
    pub fn get_codes_after_header(&self, header_text:&str) -> Vec<&CodeBlock> {
        let r = self.get_items_in_header(header_text, 2, |x| {
            if let Some(_c) = x.get_code() {
                true
            }
            else {
                false
            }
        });

        r.into_iter().map(|x| x.get_code().unwrap()).collect::<Vec<_>>()
    }

    pub fn find_code_after_header(&self, head_text:&str) -> Option<&CodeBlock> {
        let r = self.find_after_header(head_text, |x| {
            if x.get_code().is_some() {
                true
            }
            else {
                false
            }
        });

        let text_line = r?;
        let r = text_line.get_code();
        r
    }

    pub fn get_headers(&self) -> Vec<&Header> {
        self.text_line.iter()
            .filter_map(|x| x.get_header())
            .collect()
    }

    /// get header from header which is 1, 2, 3..., if give 0, it will be converted to 1
    pub fn get_headers_with_level(&self, level:u8) -> Vec<&Header> {
        let n = if level == 0 { level + 1 } else { level };

        let headers = self.get_headers();
        headers.into_iter()
            .filter(|x| x.level == n)
            .collect()
    }

    pub fn get_footnotes(&self) -> Vec<&TextLine> {
        self.text_line.iter()
            .filter_map(|x| match x { TextLine::FootNote(_, _) => Some(x), _ => None })
            .collect()
    }

    pub fn get_footnote_text_from_archor_id(&self, str:&str) -> Option<&String> {
        let foot_notes = self.get_footnotes();
        foot_notes.iter()
            .filter_map(|x| x.get_footnote())
            .find(|x| x.0 == str)
            .map(|x| x.1)
    }
}

#[derive(Debug, Clone)]
pub enum TextLine {
    BlockType(BlockType),
    HorizontalBar,
    Paragraph(Paragraph),
    Table(Table),
    FootNote(String, String),  //archer text, footnote content pair
    END,
}

impl TextLine {
    pub fn get_table(&self) -> Option<&Table> {
        match self {
            Self::Table(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_code(&self) -> Option<&CodeBlock> {
        match self {
            Self::BlockType(n) => n.get_code(),
            _ => None,
        }
    }

    pub fn get_header(&self) -> Option<&Header> {
        match self {
            Self::BlockType(n) => n.get_header(),
            _ => None,
        }
    }

    pub fn get_footnote(&self) -> Option<(&String, &String)> {
        match self {
            Self::FootNote(m, n) => Some((m, n)),
            _ => None,
        }
    }

    pub fn is_header_with_text_and_level(&self, header_text:&str, level:u8) -> bool {
        self.is_header_at_level(level) && 
        self.is_header_with_text(header_text)
    }

    pub fn is_header_at_level(&self, level:u8) -> bool {
        if let Some(h) = self.get_header() {
            h.get_level() == level
        }
        else {
            false
        }
    }

    pub fn is_header_with_text(&self, header_text:&str) -> bool {
        if let Some(h) = self.get_header() {
            &h.get_text() == header_text
        }
        else {
            false
        }
    }

    pub fn get_block(&self) -> Option<&BlockType> {
        match self {
            Self::BlockType(n) => Some(n),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Table {
    rows : Vec<Vec<RichText>>,
}

impl Table {
    pub fn rows(&self) -> &Vec<Vec<RichText>> {
        &self.rows
    }

    pub fn data_rows(&self) -> Vec<&Vec<RichText>> {
        self.rows.iter().skip(2).collect::<Vec<&Vec<_>>>()
    }

    pub fn get_demision(&self) -> Result<(usize, usize), ParsingError> {
        if let Some(row) = self.rows.first() {
            let x = row.len();
            let y = self.rows.len();
            if y<2 {
                Err(ParsingError::MarkdownPestError(MarkdownPestError::InvalidTableSize))
            }
            else {
                Ok((x, y-2))
            }
        }
        else {
            Err(ParsingError::MarkdownPestError(MarkdownPestError::InvalidTableSize))
        }
    }

    pub fn get_cell(&self, location:(usize, usize)) -> Result<&RichText, ParsingError> {
        let x = location.0;
        let y = location.1+2;  //add 2 to get cell data, if not add 2, the return value is header
        if let Some(row) = self.rows().iter().nth(y) {
            if let Some(cell) = row.iter().nth(x) {
                Ok(cell)
            }
            else {
                Err(ParsingError::MarkdownPestError(MarkdownPestError::InvalidLocation))    
            }
        }
        else {
            Err(ParsingError::MarkdownPestError(MarkdownPestError::InvalidLocation))
        }
    }

    pub fn get_col_name(&self, i:usize) -> Option<String> {
        let header = self.rows.first().unwrap();
        if i<header.len() {
            let cell = header.iter().nth(i).unwrap();
            Some(cell.get_text())
        }
        else {
            None
        }
    }

    pub fn get_col_names(&self) -> Result<Vec<String>, ParsingError> {
        let (x, _y) = self.get_demision()?;
        let mut r = Vec::default();
        for i in 0..x {
            if let Some(name) = self.get_col_name(i) {
                r.push(name.trim().to_string());
            }
            else {
                r.push(super::u32_to_base26(i as u32));
            }
        }

        Ok(r)
    }
}

pub type Paragraph = RichText;

#[derive(Debug, Clone)]
pub enum BlockType {
    Header(Header),
    Quote(Quote),
    CodeBlock(CodeBlock),
    Comment(Comment),
    MDList(MDList),
}

impl BlockType {
    pub fn get_code(&self) -> Option<&CodeBlock> {
        match self {
            Self::CodeBlock(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_header(&self) -> Option<&Header> {
        match self {
            Self::Header(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_list(&self) -> Option<&MDList> {
        match self {
            Self::MDList(n) => Some(n),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MDList {
    value : Vec<MDListItem>,
}

impl MDList {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::list => {
                let mut result = Vec::default();
                for n in pair.to_owned().into_inner() {
                    result.push(MDListItem::from_pair(&n)?);
                }

                Ok(Self { value: result })
            }
            _ => {
                error_string(format!("need to implement {pair:?} in MDList"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }

    pub fn get_value(&self) -> &Vec<MDListItem> {
        &self.value
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct MDListItem {
    rich_text: RichText,
    comments : Vec<Comment>,
}

impl MDListItem {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::list_element => {
                let inner = pair.to_owned().into_inner();
                let mut richtext_list = Vec::default();
                let mut comments = Vec::default();
                for rule in inner {
                    match rule.as_rule() {
                        Rule::rich_txt_some |
                        Rule::rich_txt => {
                            let l = rule.into_inner();
                            for n in l {
                                richtext_list.push(RichTextItem::from_pair(&n)?);
                            }
                        }
                        Rule::comment => {
                            comments.push(rule.as_str().to_string())
                        }
                        _ => {
                            error_string(format!("need to implement {pair:?} in MDListItem's innert for loop"));
                            return Err(ParsingError::ParsingConversionError)
                        }
                    }
                }

                Ok(Self { rich_text: RichText::from_vec(richtext_list), comments: comments } )
            }
            _ => {
                error_string(format!("need to implement {pair:?} in MDListItem"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }

    pub fn get_content(&self) -> &RichText {
        &self.rich_text
    }
}

pub type Comment = String;

#[derive(Debug, Clone)]
pub struct CodeBlock {
    lang : Option<String>, 
    code : String,
}

impl CodeBlock {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::codeblock => {
                let mut inner = pair.to_owned().into_inner();
                let lang = if let Some(lang_pair) = inner.find(|x| x.as_rule() == Rule::slug) {
                    Some(lang_pair.as_str().to_string())
                }
                else {
                    None
                };

                inner = pair.to_owned().into_inner();
                let mut code = String::default();
                for n in inner.filter(|x| x.as_rule() == Rule::codeblock_code) {
                    code.push_str(n.as_str());
                    code.push_str("\r\n");
                }

                Ok(Self { lang, code })                
            }
            _ => {
                error_string(format!("need to implement {pair:?} in CodeBlock"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }

    pub fn get_lang(&self) -> Option<&String> {
        self.lang.as_ref()
    }

    pub fn get_code(&self) -> &String {
        &self.code
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Quote {
    value : Vec<QuoteLine>, 
}

impl Quote {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::quote => {
                let mut result = Vec::default();
                for rule in pair.to_owned().into_inner() {
                    if rule.as_rule() == Rule::quote_line {
                        result.push(QuoteLine::from_pair(&rule)?);
                    }
                }

                Ok(Self { value: result })
            }
            _ => {
                error_string(format!("need to implement {pair:?} in Quote"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum QuoteLine {
    InlineSymbol(Inline),
    Text(String),
}

impl QuoteLine {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::inline_symbol => Ok(Self::InlineSymbol(Inline::from_pair(pair)?)),
            Rule::text => Ok(Self::Text(pair.as_str().to_string())),
            _ => {
                error_string(format!("need to implement {pair:?} in QuoteLine"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Header {
    level : u8, 
    rich_text : RichText,
}

impl Header {
    pub fn new(level:u8, rich_text:RichText) -> Self {
        Self { level, rich_text }
    }

    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        let txt = match pair.as_rule() {
            Rule::h1 |
            Rule::h2 |
            Rule::h3 |
            Rule::h4 |
            Rule::h5 |
            Rule::h6 => {
                let richtext_pair = pair.to_owned().into_inner().nth(0).unwrap();
                let r = from_pair_vec_null_fn_template(&richtext_pair, Rule::rich_txt, |rule| {
                    RichTextItem::from_pair(&rule).unwrap()
                })?;
                r
            }
            _ => {
                error_string(format!("need to implement {pair:?} in Header"));
                return Err(ParsingError::ParsingConversionError)
            }
        };

        let level = match pair.as_rule() {
            Rule::h1 => 1,
            Rule::h2 => 2,
            Rule::h3 => 3,
            Rule::h4 => 4,
            Rule::h5 => 5,
            Rule::h6 => 6,
            _ => {
                error_string(format!("need to implement {pair:?} in Header's layer"));
                return Err(ParsingError::ParsingConversionError)
            }
        };

        Ok(Self { level, rich_text: RichText::from_vec(txt) })
    }

    pub fn get_text(&self) -> String {
        let s = self.rich_text.content.iter()
            .filter_map(|x| x.get_text())
            .fold(String::default(), |accu, val| {
                format!("{accu}{val}")
            });

        s
    }

    pub fn get_level(&self) -> u8 {
        self.level
    }
}

#[derive(Debug, Clone)]
pub struct RichText {
    content : Vec<RichTextItem>,
}

impl RichText {
    pub fn from_vec(content:Vec<RichTextItem>) -> Self {
        Self { content }
    }

    pub fn get_text(&self) -> String {
        let mut r = String::new();
        for n in self.content.iter() {
            match n {
                RichTextItem::Text(s) => r = format!("{r}{s}"),
                _ => continue,
            }
        }

        r
    }

    pub fn get_inline(&self) -> Option<&Inline> {
        self.content.first()?.get_inline()
    }

    pub fn get_footnotes_archer_id(&self) -> Vec<String> {
        self.content.iter()
            .filter_map(|x| x.get_footnote_archer_id())
            .collect()
    }

    pub fn get_content(&self) -> &Vec<RichTextItem> {
        &self.content
    }

    pub fn get_text_items(&self) -> Vec<&RichTextItem> {
        let r = self.get_content().iter()
            .filter(|x| x.get_text().is_some())
            .collect::<Vec<_>>();
        r
    }

    pub fn get_inline_items(&self) -> Vec<&Inline> {
        let r = self.get_content().iter()
                .filter_map(|x| x.get_inline())
                .collect::<Vec<_>>();
        r
    }
}

#[derive(Debug, Clone)]
pub enum RichTextItem {
    InlineSymbol(Inline), 
    Text(String),
    FootNoteArcher(String),
}

impl RichTextItem {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::bold | Rule::italic | Rule::inline_code | Rule::link | Rule::reflink | Rule::refurl | Rule::image => 
                Ok(RichTextItem::InlineSymbol(Inline::from_pair(pair)?)),
            Rule::text => {
                let s = pair.as_str().to_string();
                Ok(Self::Text(s))
            }
            Rule::footnote_archer_text => 
                Ok(Self::FootNoteArcher(pair.as_str().to_string())),
            _ => {
                error_string(format!("need to implement {pair:?} in RichTextItem"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }

    pub fn get_inline(&self) -> Option<&Inline> {
        match self {
            Self::InlineSymbol(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_text(&self) -> Option<&String> {
        match self {
            Self::Text(n) => Some(n),
            _ => None,
        }
    }

    pub fn get_footnote_archer_id(&self) -> Option<String> {
        match self {
            Self::FootNoteArcher(n) => Some(n.to_string()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Inline {
    Bold(Bold), 
    Italic(Italic),
    InlineCode(InlineCode),
    Link(Link),
    RefLink(RefLink),
    RefUrl(RefUrl),
    Image(Image),
}

impl Inline {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::bold => Ok(Self::Bold(Bold::from_pair(pair)?)),
            Rule::italic => Ok(Self::Italic(Italic::from_pair(pair)?)),
            Rule::inline_code => Ok(Self::InlineCode(pair.as_str().to_string())),
            Rule::link => Ok(Self::Link(Link::from_pair(pair)?)),
            Rule::reflink => Ok(Self::RefLink(RefLink::from_pair(pair)?)),
            Rule::refurl => Ok(Self::RefUrl(RefUrl::from_pair(pair)?)),
            Rule::image => Ok(Self::Image(Image::from_pair(pair)?)),
            _ => {
                error_string(format!("need to implement {pair:?} in Inline"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }

    pub fn get_link(&self) -> Option<&Link> {
        match self {
            Self::Link(n) => Some(n),
            _ => None,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Image {
    link_text : Vec<String>,
    url : String, 
    image_tags : Vec<ImageTag>,
}

impl Image {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::image => {
                let inner = pair.to_owned().into_inner().collect::<Vec<_>>();
                let last_pair = inner.last().unwrap();
                if last_pair.as_rule() == Rule::image_tags{
                    let mut result = Vec::default();
                    for n in last_pair.to_owned().into_inner() {
                        if n.as_rule() == Rule::img_tag {
                            result.push(ImageTag::from_pair(&n)?)
                        }
                    }

                    let tags = inner.iter().take(inner.len() - 2)
                                                                        .map(|x| x.as_str().to_string())
                                                                        .collect::<Vec<_>>();

                    let url = inner.iter().nth(inner.len() - 2).unwrap().as_str().to_string();
                    Ok(Self { link_text: tags, url: url, image_tags: result })
                }
                else {
                    let url = last_pair.as_str().to_string();
                    let tags = inner.iter().take(inner.len() - 1)
                                                                        .map(|x| x.as_str().to_string())
                                                                        .collect::<Vec<_>>();
                    Ok(Self { link_text: tags, url: url, image_tags: Vec::default() }) 
                }
            }
            _ => {
                error_string(format!("need to implement {pair:?} in Image"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ImageTag {
    key : String,
    value : String,
}

impl ImageTag {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::img_tag => {
                let mut inner = pair.to_owned().into_inner();
                let image_key = inner.nth(0).unwrap();
                let image_val = inner.last().unwrap();
                Ok(Self { key : image_key.as_str().to_string(), value : image_val.as_str().to_string() })
            }
            _ => {
                error_string(format!("need to implement {pair:?} in ImageTag"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RefUrl {
    key : String, 
    url : String,
}

impl RefUrl {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::refurl => {
                let mut inners = pair.to_owned().into_inner();
                let key = inners.nth(0).unwrap();
                let url = inners.last().unwrap();
                Ok(Self { key : key.as_str().to_string(), url : url.as_str().to_string() })
            }
            _ => {
                error_string(format!("need to implement {pair:?} in RefUrl"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct RefLink {
    links : Vec<LinkText>,
    key : String,
}

impl RefLink {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        let inners = pair.to_owned().into_inner().collect::<Vec<_>>();
        match pair.as_rule() {
            Rule::reflink => {
                let link_pairs = inners.iter().take(inners.len() - 1);
                
                let mut texts = Vec::default();
                for n in link_pairs {
                    match n.as_rule() {
                        Rule::inline_symbol => texts.push(LinkText::InlineSymbol(Inline::from_pair(&n)?)),
                        Rule::link_text => texts.push(LinkText::LinkText(n.as_str().to_string())),
                        _ => {
                            error_string(format!("need to implement {pair:?} in RefLink's refLink match case"));
                            return Err(ParsingError::ParsingConversionError)
                        }
                    }
                }

                let last_pair = inners.last().unwrap();
                let url = last_pair.as_str().to_string();
                Ok(Self { links: texts, key: url })
            }
            _ => {
                error_string(format!("need to implement {pair:?} in Link"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Link {
    link_text : Vec<LinkText>,
    url : String,
}

impl Link {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        let inners = pair.to_owned().into_inner().collect::<Vec<_>>();
        match pair.as_rule() {
            Rule::link => {
                let link_pairs = inners.iter().take(inners.len() - 1);
                
                let mut texts = Vec::default();
                for n in link_pairs {
                    match n.as_rule() {
                        Rule::inline_symbol => texts.push(LinkText::InlineSymbol(Inline::from_pair(&n)?)),
                        Rule::link_text => texts.push(LinkText::LinkText(n.as_str().to_string())),
                        _ => {
                            error_string(format!("need to implement {pair:?} in Link's link match case"));
                            return Err(ParsingError::ParsingConversionError)
                        }
                    }
                }

                let last_pair = inners.last().unwrap();
                let url = last_pair.as_str().to_string();
                Ok(Self { link_text: texts, url: url })
            }
            _ => {
                error_string(format!("need to implement {pair:?} in Link"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }

    pub fn get_url(&self) -> &String {
        &self.url
    }

    pub fn get_link_text(&self) -> String {
        self.link_text.iter()
            .filter_map(|x| x.get_text())
            .fold(String::new(), |acc, val| { format!("{acc}{val}") })
    }
}

#[derive(Debug, Clone)]
pub enum LinkText {
    InlineSymbol(Inline), 
    LinkText(String),
}

impl LinkText {
    pub fn get_text(&self) -> Option<&String> {
        match self {
            Self::LinkText(n) => Some(n),
            _ => None,
        }
    }
}

pub type InlineCode = String;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Italic {
    value : Vec<ItatlicContent>,
}

impl Italic {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        let v = from_pair_vec_null_fn_template(pair, Rule::italic, |rule| {
            match rule.as_rule() {
                Rule::bold | 
                Rule::inline_code |
                Rule::link |
                Rule::reflink |
                Rule::image |
                Rule::italic_text => ItatlicContent::from_pair(&rule).unwrap(),
                _ => {
                    error_string(format!("need to implement {rule:?} in Italic"));
                    todo!();
                }
            }
        })?;

        Ok(Self { value: v })
    }
}

#[derive(Debug, Clone)]
pub enum ItatlicContent {
    Bold(Bold),
    InlineCode(InlineCode),
    Link(Link),
    RefLink(RefLink),
    Image(Image),
    Text(String),
}

impl ItatlicContent {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::bold => Ok(Self::Bold(Bold::from_pair(pair)?)),
            Rule::inline_code => Ok(Self::InlineCode(pair.as_str().to_string())),
            Rule::link => Ok(Self::Link(Link::from_pair(pair)?)),
            Rule::reflink => Ok(Self::RefLink(RefLink::from_pair(pair)?)),
            Rule::image => Ok(Self::Image(Image::from_pair(pair)?)),
            Rule::italic_text => Ok(Self::Text(pair.as_str().to_string())),
            _ => {
                error_string(format!("need to implement {pair:?} in ItatlicContent"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Bold {
    value : Vec<BoldContent>,
}

impl Bold {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::bold => {
                let l = from_pair_vec_null_fn_template(pair, Rule::bold, |rule| {
                    BoldContent::from_pair(&rule).unwrap()
                })?;
                Ok(Self { value : l })
            }
            _ => {
                error_string(format!("need to implement {pair:?} in Bold"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum BoldContent {
    Italic(Italic),
    InlineCode(InlineCode),
    Link(Link),
    RefLink(RefLink),
    Image(Image),
    Text(String),
}

impl BoldContent {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, ParsingError> {
        match pair.as_rule() {
            Rule::italic => Ok(Self::Italic(Italic::from_pair(pair)?)),
            Rule::inline_code => Ok(Self::InlineCode(pair.as_str().to_string())),
            Rule::link => Ok(Self::Link(Link::from_pair(pair)?)),
            Rule::reflink => Ok(Self::RefLink(RefLink::from_pair(pair)?)),
            Rule::image => Ok(Self::Image(Image::from_pair(pair)?)),
            Rule::bold_text => {
                let s = pair.as_str().to_string();
                Ok(Self::Text(s))
            }
            _ => {
                error_string(format!("need to implement {pair:?} in BoldContent"));
                Err(ParsingError::ParsingConversionError)
            }
        }
    }
}

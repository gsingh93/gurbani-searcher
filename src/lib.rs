extern crate rusqlite;
extern crate deuterium;

#[macro_use]
extern crate log;

use deuterium::*;

use rusqlite::{SqliteConnection, SqliteRows, SqliteRow, SqliteStatement};
use rusqlite::types::ToSql as ToSqlite;

use std::convert::From as StdFrom;
use std::path::Path;

const DB_PATH: &'static str = "resources/gurbani.db";
const TABLE_NAME: &'static str = "scriptures";

#[derive(Debug)]
pub enum Scripture {
    SGGS
}

impl Scripture {
    fn name(&self) -> &'static str {
        match self {
            &Scripture::SGGS => "sggs"
        }
    }
}

#[derive(Debug)]
pub struct QueryParams {
    pub scripture: Option<Scripture>,
    pub page: Option<i16>,
    pub hymn: Option<i16>,
    pub gurmukhi: Option<String>,
    pub transliteration: Option<String>
}

impl QueryParams {
    pub fn new() -> Self {
        QueryParams { scripture: None, page: None, hymn: None, gurmukhi: None,
                      transliteration: None }
    }

    pub fn scripture(mut self, scripture: Scripture) -> Self {
        self.scripture = Some(scripture);
        self
    }

    pub fn page(mut self, page: i16) -> Self {
        self.page = Some(page);
        self
    }

    pub fn hymn(mut self, hymn: i16) -> Self {
        self.hymn = Some(hymn);
        self
    }

    pub fn gurmukhi(mut self, gurmukhi: String) -> Self {
        self.gurmukhi = Some(gurmukhi);
        self
    }

    pub fn transliteration(mut self, transliteration: String) -> Self {
        self.transliteration = Some(transliteration);
        self
    }
}

#[derive(Debug)]
pub struct Record {
    pub id: i32,
    pub scripture: String,
    pub page: i64,
    pub line: i64,
    pub hymn: i64,
    pub gurmukhi: String,
    pub transliteration: String,
    pub translation: String,
    pub attributes: String,
    pub gurmukhi_search: String,
    pub transliteration_search: String
}

pub struct DbConnection(SqliteConnection);

impl DbConnection {
    pub fn connect() -> DbConnection {
        DbConnection(SqliteConnection::open(&Path::new(DB_PATH)).unwrap())
    }

    pub fn query<'a>(&'a self, params: QueryParams) -> Stmt<'a> {
        let (sql, args) = construct_query(params);
        debug!("{}", sql);
        let stmt = Stmt { stmt: self.0.prepare(&sql).unwrap(), args: args };
        stmt
    }
}

pub struct Stmt<'conn> {
    stmt: SqliteStatement<'conn>,
    args: Vec<Box<ToSqlite>>
}

impl<'conn> Stmt<'conn> {
    pub fn query<'a>(&'a mut self) -> Rows<'a> {
        let args_ref: Vec<_> = self.args.iter().map(|x| &**x as &ToSqlite).collect();
        Rows { rows: self.stmt.query(&*args_ref).unwrap() }
    }
}

pub struct Rows<'stmt> {
    rows: SqliteRows<'stmt>
}

pub struct Row<'stmt> {
    row: SqliteRow<'stmt>
}

impl<'stmt> Row<'stmt> {
    pub fn to_record(&self) -> Record {
        Record {
            id: self.id(),
            scripture: self.scripture(),
            page: self.page(),
            line: self.line(),
            hymn: self.hymn(),
            gurmukhi: self.gurmukhi(),
            transliteration: self.transliteration(),
            translation: self.translation(),
            attributes: self.attributes(),
            gurmukhi_search: self.gurmukhi_search(),
            transliteration_search: self.transliteration_search(),
        }
    }

    pub fn id(&self) -> i32 {
        self.row.get_checked(0).unwrap()
    }

    pub fn scripture(&self) -> String {
        self.row.get_checked(1).unwrap()
    }

    pub fn page(&self) -> i64 {
        self.row.get_checked(2).unwrap()
    }

    pub fn line(&self) -> i64 {
        self.row.get_checked(3).unwrap()
    }

    pub fn hymn(&self) -> i64 {
        self.row.get_checked(4).unwrap()
    }

    pub fn gurmukhi(&self) -> String {
        self.row.get_checked(5).unwrap()
    }

    pub fn transliteration(&self) -> String {
        self.row.get_checked(6).unwrap()
    }

    pub fn translation(&self) -> String {
        self.row.get_checked(7).unwrap()
    }

    pub fn attributes(&self) -> String {
        self.row.get_checked(8).unwrap()
    }

    pub fn gurmukhi_search(&self) -> String {
        self.row.get_checked(9).unwrap()
    }

    pub fn transliteration_search(&self) -> String {
        self.row.get_checked(10).unwrap()
    }

}

impl<'stmt> StdFrom<SqliteRow<'stmt>> for Row<'stmt> {
    fn from(row: SqliteRow<'stmt>) -> Row<'stmt> {
        Row { row: row }
    }
}

impl<'stmt> Iterator for Rows<'stmt> {
    type Item = Row<'stmt>;

    fn next(&mut self) -> Option<Row<'stmt>> {
        self.rows.next().map(|x| x.unwrap().into())
    }
}

fn construct_query(params: QueryParams) -> (String, Vec<Box<ToSqlite>>) {
    let table = TableDef::new(TABLE_NAME);

    let mut args: Vec<Box<ToSqlite>> = vec!();
    let mut query = table.select_all();

    if let Some(ref scripture) = params.scripture {
        let scripture_column = NamedField::<String>::field_of("scripture", &table);
        query = query.where_(scripture_column.is(scripture.name().to_string()));
        args.push(Box::new(scripture.name()));
    }
    if let Some(page) = params.page {
        let page_column = NamedField::<i16>::field_of("page", &table);
        query = query.where_(page_column.is(page));
        args.push(Box::new(page as i64));
    }
    if let Some(hymn) = params.hymn {
        let hymn_column = NamedField::<i16>::field_of("hymn", &table);
        query = query.where_(hymn_column.is(hymn));
        args.push(Box::new(hymn as i64));
    }
    if let Some(ref gurmukhi) = params.gurmukhi {
        let gurmukhi_column = NamedField::<String>::field_of("gurmukhi_search", &table);
        let mut gurmukhi = gurmukhi.to_owned();
        gurmukhi.push('%');
        query = query.where_(gurmukhi_column.like(gurmukhi.clone()));
        args.push(Box::new(gurmukhi));
    }
    if let Some(ref transliteration) = params.transliteration {
        let transliteration_column = NamedField::<String>::field_of("transliteration_search",
                                                                    &table);
        let mut transliteration = transliteration.to_owned();
        transliteration.push('%');
        query = query.where_(transliteration_column.is(transliteration.clone()));
        args.push(Box::new(transliteration));
    }

    let mut context = SqlContext::new(Box::new(sql::PostgreSqlAdapter));
    (query.to_final_sql(&mut context), args)
}

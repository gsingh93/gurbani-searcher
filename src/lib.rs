extern crate rusqlite;
extern crate deuterium;

use deuterium::{sql, TableDef, NamedField, SqlContext, ToIsPredicate, Selectable, Queryable,
                QueryToSql};

use rusqlite::SqliteConnection;
use rusqlite::types::ToSql;

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
    pub page: Option<i16>
}

#[derive(Debug)]
pub struct QueryResult {
    id: i32,
    scripture: String,
    page: i64,
    line: i64,
    hymn: i64,
    pub gurmukhi: String,
    transliteration: String,
    translation: String,
    attributes: String,
    gurmukhi_search: String,
    transliteration_search: String
}

pub fn connect() -> SqliteConnection {
    SqliteConnection::open(&Path::new(DB_PATH)).unwrap()
}

pub fn query(conn: &SqliteConnection, params: QueryParams) -> Vec<QueryResult> {
    let (query, args) = construct_query(params);
    let args_ref: Vec<_> = args.iter().map(|x| &**x as &ToSql).collect();
    let mut stmt = conn.prepare(&query).unwrap();
    let mut results: Vec<QueryResult> = vec!();
    for row in stmt.query(&*args_ref).unwrap().map(|row| row.unwrap()) {
        let translation: String = row.get(7);
        let attributes: String = row.get(8);
        let gurmukhi_search: String = row.get(9);
        let transliteration_search: String = row.get(10);

        let res = QueryResult {
            id: row.get(0),
            scripture: row.get(1),
            page: row.get(2),
            line: row.get(3),
            hymn: row.get(4),
            gurmukhi: row.get(5),
            transliteration: row.get(6),
            translation: translation,
            attributes: attributes,
            gurmukhi_search: gurmukhi_search,
            transliteration_search: transliteration_search,
        };
        results.push(res);
    }
    results
}

fn construct_query(params: QueryParams) -> (String, Vec<Box<ToSql>>) {
    let table = TableDef::new(TABLE_NAME);

    let mut args: Vec<Box<ToSql>> = vec!();
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

    let mut context = SqlContext::new(Box::new(sql::PostgreSqlAdapter));
    (query.to_final_sql(&mut context), args)
}

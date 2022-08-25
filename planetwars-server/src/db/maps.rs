use diesel::{PgConnection, QueryDsl, QueryResult, RunQueryDsl};

use crate::schema::maps;

#[derive(Insertable)]
#[table_name = "maps"]
pub struct NewMap<'a> {
    pub name: &'a str,
    pub file_path: &'a str,
}

#[derive(Queryable, Clone, Debug)]
pub struct Map {
    pub id: i32,
    pub name: String,
    pub file_path: String,
}

pub fn create_map(new_map: NewMap, conn: &PgConnection) -> QueryResult<Map> {
    diesel::insert_into(maps::table)
        .values(new_map)
        .get_result(conn)
}

pub fn find_map(id: i32, conn: &PgConnection) -> QueryResult<Map> {
    maps::table.find(id).get_result(conn)
}

pub fn list_maps(conn: &PgConnection) -> QueryResult<Vec<Map>> {
    maps::table.get_results(conn)
}
use diesel::prelude::*;

use crate::schema::maps;

#[derive(Insertable)]
#[diesel(table_name = maps)]
pub struct NewMap<'a> {
    pub name: &'a str,
    pub file_path: &'a str,
}

#[derive(Queryable, Clone, Debug)]
pub struct Map {
    pub id: i32,
    pub name: String,
    pub file_path: String,
    pub is_ranked: bool,
}

pub fn create_map(new_map: NewMap, conn: &mut PgConnection) -> QueryResult<Map> {
    diesel::insert_into(maps::table)
        .values(new_map)
        .get_result(conn)
}

pub fn find_map(id: i32, conn: &mut PgConnection) -> QueryResult<Map> {
    maps::table.find(id).get_result(conn)
}

pub fn find_map_by_name(name: &str, conn: &mut PgConnection) -> QueryResult<Map> {
    maps::table.filter(maps::name.eq(name)).first(conn)
}

pub fn list_maps(conn: &mut PgConnection) -> QueryResult<Vec<Map>> {
    maps::table.get_results(conn)
}

pub fn get_ranked_maps(conn: &mut PgConnection) -> QueryResult<Vec<Map>> {
    maps::table
        .filter(maps::is_ranked.eq(true))
        .get_results(conn)
}

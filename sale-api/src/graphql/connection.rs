use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::OutputType;
use sale::infra::aws::ddb::cursor::EntityWithCursor;

pub fn connection_from<T, U: OutputType>(
    from: Vec<EntityWithCursor<T>>,
    conv: impl Fn(T) -> U,
) -> Connection<String, U> {
    let has_next = !from.is_empty();
    let mut edges = from
        .into_iter()
        .map(|product| {
            Edge::<String, U, EmptyFields>::new(product.cursor.to_string(), conv(product.entity))
        })
        .collect::<Vec<_>>();
    let mut connection = Connection::new(false, has_next);
    connection.edges.append(&mut edges);
    connection
}

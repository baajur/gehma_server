/*pub struct Event {
    pub id: uuid::Uuid,
    pub name: String,
    pub description: String,
    pub opening: chrono::NaiveDateTime,
    pub closing: chrono::NaiveDateTime,
    pub country: String,
    pub city: String,
    pub addr: String,
    pub link: String,
    pub created_at: chrono::NaiveDateTime,
    pub changed_at: chrono::NaiveDateTime,
}
*/

CREATE TABLE events (
    id UUID NOT NULL PRIMARY KEY,
    name VARCHAR(128) NOT NULL,
    description VARCHAR(16384) NOT NULL,
    opening TIMESTAMP NOT NULL,
    closing TIMESTAMP NOT NULL,
    country VARCHAR(32) NOT NULL,
    city VARCHAR(32) NOT NULL,
    addr VARCHAR(128) NOT NULL,
    href VARCHAR(1024),
    created_at TIMESTAMP NOT NULL,
    changed_at TIMESTAMP NOT NULL
)

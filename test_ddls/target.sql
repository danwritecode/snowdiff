CREATE TABLE users (
    user_id INTEGER AUTOINCREMENT PRIMARY KEY,
    username VARCHAR(50) NOT NULL,
    email VARCHAR(100) UNIQUE NOT NULL,
    created_at TIMESTAMP_NTZ DEFAULT CURRENT_TIMESTAMP
);
CREATE TABLE products (
    product_id INTEGER AUTOINCREMENT PRIMARY KEY,
    name VARCHAR(100),
    metadata VARIANT
);

CREATE TABLE raw_events_staging (
    event_id STRING,
    event_payload VARIANT,
    received_at TIMESTAMP_NTZ
);

CREATE VIEW active_users AS
SELECT
    user_id,
    email
FROM users
WHERE status = 'active';

CREATE VIEW user_order_summary AS
SELECT
    u.user_id,
    u.username,
    COUNT(o.order_id) AS total_orders,
    SUM(o.order_total) AS total_spent
FROM users u
LEFT JOIN orders o ON u.user_id = o.user_id
GROUP BY u.user_id, u.username;

CREATE VIEW product_metadata_view AS
SELECT
    metadata:"category"::STRING AS category,
    metadata:"tags"::ARRAY AS tags
FROM products;

CREATE TABLE sensor_readings (
    sensor_id STRING,
    reading_time TIMESTAMP_NTZ
)

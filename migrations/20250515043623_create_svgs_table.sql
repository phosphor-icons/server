-- Create a table that stores the SVG source code of each icon variant
CREATE TABLE svgs (
    id SERIAL PRIMARY KEY,
    icon_id INTEGER NOT NULL REFERENCES icons(id) ON DELETE CASCADE,
    weight TEXT NOT NULL,
    src TEXT NOT NULL
);

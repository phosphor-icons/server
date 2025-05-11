-- Enable the uuid-ossp extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create the icons table
CREATE TABLE icons (
    id SERIAL PRIMARY KEY,
    rid TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    status TEXT NOT NULL,
    category TEXT NOT NULL,
    search_categories TEXT[] DEFAULT '{}',
    tags TEXT[] DEFAULT '{}',
    notes TEXT,
    released_at FLOAT,
    last_updated_at FLOAT,
    deprecated_at FLOAT,
    published BOOLEAN DEFAULT FALSE,
    alias TEXT,
    code INTEGER UNIQUE CHECK (
        (code BETWEEN x'E000'::int AND x'F8FF'::int OR code BETWEEN x'F0000'::int AND x'FFFFD'::int) AND
        code % 2 = 0
    ));

-- Codepoint assignment
CREATE OR REPLACE FUNCTION assign_code_point()
RETURNS TRIGGER AS $$
DECLARE
    next_cp INTEGER;
BEGIN
    IF NEW.code IS NOT NULL THEN
        RETURN NEW;
    END IF;

    -- Try from BMP private use range first
    SELECT cp INTO next_cp FROM (
        SELECT generate_series(x'E000'::int, x'F8FF'::int, 2) AS cp
        EXCEPT
        SELECT code FROM icons WHERE code BETWEEN x'E000'::int AND x'F8FF'::int
    ) AS free_codes
    ORDER BY cp
    LIMIT 1;

    -- If none found in BMP, try the Supplementary PUA range
    IF next_cp IS NULL THEN
        SELECT cp INTO next_cp FROM (
            SELECT generate_series(x'F0000'::int, x'FFFFD'::int, 2) AS cp
            EXCEPT
            SELECT code FROM icons WHERE code BETWEEN x'F0000'::int AND x'FFFFD'::int
        ) AS free_codes
        ORDER BY cp
        LIMIT 1;
    END IF;

    IF next_cp IS NULL THEN
        RAISE EXCEPTION 'No available code points in defined private use ranges';
    END IF;

    NEW.code := next_cp;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_assign_code_point
BEFORE INSERT ON icons
FOR EACH ROW
EXECUTE FUNCTION assign_code_point();

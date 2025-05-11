-- Make `tags` column non-nullable
ALTER TABLE icons
    ALTER COLUMN tags SET NOT NULL,
    ALTER COLUMN tags SET DEFAULT '{}';

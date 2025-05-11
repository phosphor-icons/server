-- Make `published ` column non-nullable
ALTER TABLE icons
    ALTER COLUMN published SET NOT NULL,
    ALTER COLUMN published SET DEFAULT FALSE;

-- Make `search_categories` column non-nullable
ALTER TABLE icons
    ALTER COLUMN search_categories SET NOT NULL,
    ALTER COLUMN search_categories SET DEFAULT '{}';

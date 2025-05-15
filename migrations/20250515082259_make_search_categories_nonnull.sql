-- Make combinations of `icon_id` and `weight` unique
ALTER TABLE svgs ADD CONSTRAINT unique_icon_weight UNIQUE (icon_id, weight);

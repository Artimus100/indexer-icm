-- Drop the holders table and its associated objects
DROP TRIGGER IF EXISTS update_holders_updated_at ON holders;
DROP TABLE IF EXISTS holders;
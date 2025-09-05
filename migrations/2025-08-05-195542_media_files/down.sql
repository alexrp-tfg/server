-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS "idx_media_files_uploaded_at";
DROP INDEX IF EXISTS "idx_media_files_user_id";
DROP TABLE IF EXISTS "media_files";

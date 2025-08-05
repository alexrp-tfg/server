-- Your SQL goes here
CREATE TABLE IF NOT EXISTS "media_files" (
    "id" UUID DEFAULT gen_random_uuid() PRIMARY KEY,
    "user_id" UUID NOT NULL REFERENCES "users"("id") ON DELETE CASCADE,
    "filename" VARCHAR(255) NOT NULL,
    "original_filename" VARCHAR(255) NOT NULL,
    "file_size" BIGINT NOT NULL,
    "content_type" VARCHAR(100) NOT NULL,
    "file_path" VARCHAR(500) NOT NULL,
    "uploaded_at" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    "updated_at" TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS "idx_media_files_user_id" ON "media_files"("user_id");
CREATE INDEX IF NOT EXISTS "idx_media_files_uploaded_at" ON "media_files"("uploaded_at");

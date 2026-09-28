ALTER TABLE llm_web_relay_jobs
  ADD COLUMN IF NOT EXISTS request_hash text;

UPDATE llm_web_relay_jobs
   SET request_hash = 'legacy-md5:' || md5(
       backend || chr(0) || model || chr(0) || system_prompt || chr(0) ||
       user_prompt || chr(0) || response_format || chr(0) ||
       allow_tools::text
   )
 WHERE request_hash IS NULL;

CREATE INDEX IF NOT EXISTS idx_llm_web_relay_request_hash
  ON llm_web_relay_jobs(request_hash);

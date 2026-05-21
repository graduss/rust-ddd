-- ============================================================
--
-- Схема для хранения агрегата Task. Намеренно плоская —
-- DDD не требует прямого мэппинга агрегат → таблица,
-- мы делаем так для простоты примера.
-- ============================================================

CREATE TYPE task_status AS ENUM ('todo', 'in_progress', 'completed', 'cancelled');

CREATE TABLE IF NOT EXISTS tasks (
    id          UUID         PRIMARY KEY,
    title       VARCHAR(255) NOT NULL,
    description TEXT         NOT NULL DEFAULT '',
    status      task_status  NOT NULL DEFAULT 'todo',
    created_at  TIMESTAMPTZ  NOT NULL,
    updated_at  TIMESTAMPTZ  NOT NULL,
);

CREATE INDEX IF NOT EXISTS idx_tasks_status     ON tasks (status);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks (created_at DESC);

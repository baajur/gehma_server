CREATE OR REPLACE VIEW stat_monthly AS (
    SELECT tele_num, date_part('month', usage_statistics.created_at::date) as monthly, date_part('year', usage_statistics.created_at::date) as yearly, COUNT(usage_statistics.created_at::date), (dense_rank() over (ORDER BY date_part('year', usage_statistics.created_at::date) ASC, date_part('month', usage_statistics.created_at::date) ASC)) as period FROM usage_statistics  GROUP BY tele_num, monthly, yearly ORDER
    BY yearly ASC, monthly ASC
);

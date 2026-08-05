<?php

declare(strict_types=1);

namespace Core;

use PDO;

class Query
{
    private string $table;
    private array $wheres = [];
    private array $bindings = [];
    private string $fields = '*';
    private array $orders = [];
    private ?int $limit = null;
    private ?int $offset = null;
    private array $data = [];
    private int $bindIndex = 0;

    public function __construct(string $table)
    {
        $this->table = $table;
    }

    public function field(string $fields): self
    {
        $this->fields = $fields;
        return $this;
    }

    public function where(mixed $field, mixed $op = null, mixed $condition = null, string $logic = 'AND'): self
    {
        if (is_array($field)) {
            foreach ($field as $k => $v) {
                $this->where($k, '=', $v, $logic);
            }
            return $this;
        }

        if ($condition === null && $op !== null && !in_array(strtolower((string) $op), [
            '=', '<>', '!=', '<', '>', '<=', '>=', 'like', 'not like', 'in', 'not in', 'between', 'between time', 'null', 'not null',
        ], true)) {
            $condition = $op;
            $op = '=';
        } elseif ($condition === null && $op === null) {
            $condition = null;
            $op = '=';
        }

        $op = strtolower((string) ($op ?? '='));

        if ($op === 'between time') {
            return $this->whereBetweenTime((string) $field, (array) $condition, $logic);
        }
        if ($op === 'between') {
            return $this->whereBetween((string) $field, (array) $condition, $logic, false);
        }
        if ($op === 'in' || $op === 'not in') {
            $values = is_array($condition) ? $condition : explode(',', (string) $condition);
            if ($values === []) {
                $this->wheres[] = [$logic, '1 = 0'];
                return $this;
            }
            $placeholders = [];
            foreach ($values as $value) {
                $placeholders[] = $this->bind($value);
            }
            $this->wheres[] = [$logic, sprintf('`%s` %s (%s)', $field, strtoupper($op), implode(',', $placeholders))];
            return $this;
        }

        $placeholder = $this->bind($condition);
        $this->wheres[] = [$logic, sprintf('`%s` %s %s', $field, strtoupper($op), $placeholder)];
        return $this;
    }

    public function whereTime(string $field, string $op, mixed $range = null): self
    {
        $op = strtolower($op);
        if ($op === 'between' && is_array($range)) {
            return $this->whereBetweenTime($field, $range, 'AND');
        }

        [$start, $end] = $this->timeRange($op, $range);
        return $this->whereBetween($field, [$start, $end], 'AND', true);
    }

    public function order(string $field, string $order = 'asc'): self
    {
        if (str_contains($field, ' ')) {
            $this->orders[] = $field;
        } else {
            $this->orders[] = sprintf('`%s` %s', $field, strtoupper($order));
        }
        return $this;
    }

    public function limit(int $limit, ?int $offset = null): self
    {
        $this->limit = $limit;
        if ($offset !== null) {
            $this->offset = $offset;
        }
        return $this;
    }

    public function page(int|string $page, ?int $listRows = null): self
    {
        $page = max(1, (int) $page);
        if ($listRows !== null) {
            $this->limit = $listRows;
        }
        $rows = $this->limit ?? 20;
        $this->offset = ($page - 1) * $rows;
        $this->limit = $rows;
        return $this;
    }

    public function data(array $data): self
    {
        $this->data = $data;
        return $this;
    }

    public function insert(?array $data = null): int|string
    {
        $data = $data ?? $this->data;
        if ($data === []) {
            return 0;
        }
        $columns = [];
        $placeholders = [];
        foreach ($data as $key => $value) {
            $columns[] = '`' . $key . '`';
            $placeholders[] = $this->bind($value);
        }
        $sql = sprintf(
            'INSERT INTO `%s` (%s) VALUES (%s)',
            $this->table,
            implode(',', $columns),
            implode(',', $placeholders)
        );
        $this->run($sql);
        return Db::pdo()->lastInsertId();
    }

    public function update(array $data): int
    {
        if ($data === []) {
            return 0;
        }
        $sets = [];
        foreach ($data as $key => $value) {
            $sets[] = sprintf('`%s` = %s', $key, $this->bind($value));
        }
        $sql = sprintf('UPDATE `%s` SET %s%s', $this->table, implode(',', $sets), $this->buildWhere());
        return $this->run($sql);
    }

    public function delete(): int
    {
        $sql = sprintf('DELETE FROM `%s`%s', $this->table, $this->buildWhere());
        return $this->run($sql);
    }

    public function setInc(string $field, int $step = 1): int
    {
        $sql = sprintf(
            'UPDATE `%s` SET `%s` = `%s` + %d%s',
            $this->table,
            $field,
            $field,
            $step,
            $this->buildWhere()
        );
        return $this->run($sql);
    }

    public function setDec(string $field, int $step = 1): int
    {
        return $this->setInc($field, -$step);
    }

    public function find(): ?array
    {
        $this->limit = 1;
        $rows = $this->select();
        return $rows[0] ?? null;
    }

    public function value(string $field): mixed
    {
        $this->fields = '`' . str_replace('`', '', $field) . '`';
        $row = $this->find();
        return $row[$field] ?? null;
    }

    public function column(string $field, ?string $key = null): array
    {
        $rows = $this->select();
        if ($key === null) {
            return array_column($rows, $field);
        }
        return array_column($rows, $field, $key);
    }

    public function count(string $field = '*'): int
    {
        return (int) $this->aggregate('COUNT', $field);
    }

    public function min(string $field): mixed
    {
        return $this->aggregate('MIN', $field);
    }

    public function max(string $field): mixed
    {
        return $this->aggregate('MAX', $field);
    }

    public function sum(string $field): mixed
    {
        return $this->aggregate('SUM', $field);
    }

    public function select(): array
    {
        $sql = sprintf(
            'SELECT %s FROM `%s`%s%s%s',
            $this->fields,
            $this->table,
            $this->buildWhere(),
            $this->orders ? ' ORDER BY ' . implode(', ', $this->orders) : '',
            $this->buildLimit()
        );
        $stmt = Db::pdo()->prepare($sql);
        $stmt->execute($this->bindings);
        return $stmt->fetchAll(PDO::FETCH_ASSOC) ?: [];
    }

    private function aggregate(string $func, string $field): mixed
    {
        $expr = $field === '*' ? '*' : '`' . str_replace('`', '', $field) . '`';
        $sql = sprintf('SELECT %s(%s) AS `aggregate` FROM `%s`%s', $func, $expr, $this->table, $this->buildWhere());
        $stmt = Db::pdo()->prepare($sql);
        $stmt->execute($this->bindings);
        $row = $stmt->fetch(PDO::FETCH_ASSOC);
        return $row['aggregate'] ?? null;
    }

    private function run(string $sql): int
    {
        $stmt = Db::pdo()->prepare($sql);
        $stmt->execute($this->bindings);
        return $stmt->rowCount();
    }

    private function buildWhere(): string
    {
        if ($this->wheres === []) {
            return '';
        }
        $parts = [];
        foreach ($this->wheres as $i => [$logic, $expr]) {
            $parts[] = ($i === 0 ? '' : ' ' . $logic . ' ') . $expr;
        }
        return ' WHERE ' . implode('', $parts);
    }

    private function buildLimit(): string
    {
        if ($this->limit === null) {
            return '';
        }
        if ($this->offset !== null) {
            return sprintf(' LIMIT %d, %d', $this->offset, $this->limit);
        }
        return sprintf(' LIMIT %d', $this->limit);
    }

    private function bind(mixed $value): string
    {
        $key = ':b' . $this->bindIndex++;
        $this->bindings[$key] = $value;
        return $key;
    }

    private function whereBetween(string $field, array $range, string $logic, bool $numeric): self
    {
        $start = $range[0] ?? null;
        $end = $range[1] ?? null;
        if ($numeric) {
            $start = (int) $start;
            $end = (int) $end;
        }
        $this->wheres[] = [
            $logic,
            sprintf('`%s` BETWEEN %s AND %s', $field, $this->bind($start), $this->bind($end)),
        ];
        return $this;
    }

    private function whereBetweenTime(string $field, array $range, string $logic): self
    {
        $start = $this->toTimestamp($range[0] ?? null, false);
        $end = $this->toTimestamp($range[1] ?? null, true);
        if ($start === $end) {
            $end = $start + 86399;
        }
        return $this->whereBetween($field, [$start, $end], $logic, true);
    }

    private function timeRange(string $op, mixed $range = null): array
    {
        $now = time();
        return match ($op) {
            'today' => [strtotime('today'), strtotime('tomorrow') - 1],
            'yesterday' => [strtotime('yesterday'), strtotime('today') - 1],
            'week' => [strtotime('monday this week'), strtotime('monday next week') - 1],
            'last week' => [strtotime('monday last week'), strtotime('monday this week') - 1],
            'month' => [strtotime(date('Y-m-01 00:00:00')), strtotime(date('Y-m-01 00:00:00', strtotime('+1 month'))) - 1],
            'last month' => [strtotime(date('Y-m-01 00:00:00', strtotime('first day of last month'))), strtotime(date('Y-m-01 00:00:00')) - 1],
            'year' => [strtotime(date('Y-01-01 00:00:00')), strtotime(date('Y-01-01 00:00:00', strtotime('+1 year'))) - 1],
            default => is_array($range)
                ? [$this->toTimestamp($range[0] ?? $now, false), $this->toTimestamp($range[1] ?? $now, true)]
                : [$this->toTimestamp($range, false), $this->toTimestamp($range, true)],
        };
    }

    private function toTimestamp(mixed $value, bool $endOfDay): int
    {
        if ($value === null || $value === '') {
            return time();
        }
        if (is_numeric($value)) {
            return (int) $value;
        }
        $value = (string) $value;
        if (preg_match('/^\d{2}-\d{2}$/', $value)) {
            $value = date('Y') . '-' . $value;
        }
        $ts = strtotime($value);
        if ($ts === false) {
            return time();
        }
        if ($endOfDay && !preg_match('/\d{2}:\d{2}/', $value)) {
            return $ts + 86399;
        }
        return $ts;
    }
}

<?php

declare(strict_types=1);

namespace think;

use Core\Controller as CoreController;

class Controller extends CoreController
{
    /** ThinkPHP chain helper used as ->remember(); no-op. */
    public function remember(): static
    {
        return $this;
    }

    public function redirect(string $url): static
    {
        // Keep fluent API used by old code: $this->redirect(...)->remember();
        $target = preg_match('#^https?://#i', $url) ? $url : url($url);
        header('Location: ' . $target);
        // Do not exit yet so ->remember() can be called; callers usually exit afterwards.
        // To preserve behavior when not chaining, flush and exit on shutdown if still here.
        register_shutdown_function(static function () {
            if (!headers_sent()) {
                // already sent Location
            }
            exit;
        });
        return $this;
    }
}

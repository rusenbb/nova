/*
 * Nova Core Library - C FFI Header
 *
 * This header provides the C interface for native frontends
 * (Swift, GTK4, WinUI) to interact with the Nova search engine.
 *
 * All complex data types are returned as JSON strings.
 * Use nova_string_free() to free any strings returned by these functions.
 */

#ifndef NOVA_CORE_H
#define NOVA_CORE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Opaque handle to the Nova core engine.
 *
 * This struct holds all the state needed for search and execution.
 * It is created by nova_core_new() and must be freed with nova_core_free().
 */
typedef struct NovaCore NovaCore;

/**
 * Create a new Nova core instance.
 *
 * Returns a pointer to the core instance, or NULL on failure.
 * The caller is responsible for calling nova_core_free() to release the memory.
 */
NovaCore* nova_core_new(void);

/**
 * Free a Nova core instance.
 *
 * The handle must be a valid pointer returned by nova_core_new().
 * After calling this function, the handle is no longer valid.
 */
void nova_core_free(NovaCore* handle);

/**
 * Perform a search and return JSON results.
 *
 * @param handle A valid NovaCore handle from nova_core_new()
 * @param query The search query as a C string (UTF-8)
 * @param max_results Maximum number of results to return
 *
 * @return A JSON string containing the search results. The caller must free
 *         this string using nova_string_free(). Returns NULL on error.
 *
 * JSON format:
 * {
 *   "results": [
 *     {"type": "App", "data": {"id": "...", "name": "...", ...}},
 *     {"type": "Calculation", "data": {"expression": "...", "result": "..."}},
 *     ...
 *   ]
 * }
 */
char* nova_core_search(NovaCore* handle, const char* query, uint32_t max_results);

/**
 * Execute a search result by index.
 *
 * @param handle A valid NovaCore handle
 * @param index Index of the result in the last search results (0-based)
 *
 * @return A JSON string containing the execution result. The caller must free
 *         this string using nova_string_free(). Returns NULL on error.
 *
 * JSON format:
 * {"result": "Success"}
 * {"result": "Error", "message": "..."}
 * {"result": "NeedsInput"}
 * {"result": "OpenSettings"}
 * {"result": "Quit"}
 */
char* nova_core_execute(NovaCore* handle, uint32_t index);

/**
 * Poll the clipboard for new content.
 *
 * Call this periodically to update the clipboard history.
 */
void nova_core_poll_clipboard(NovaCore* handle);

/**
 * Reload configuration and refresh app list.
 */
void nova_core_reload(NovaCore* handle);

/**
 * Get the number of results from the last search.
 */
uint32_t nova_core_result_count(NovaCore* handle);

/**
 * Free a string allocated by the FFI functions.
 *
 * The pointer must be a valid string returned by one of the FFI functions,
 * or NULL (which is safely ignored).
 */
void nova_string_free(char* ptr);

// ============================================================================
// Theme API
// ============================================================================

/**
 * Get the complete theme as a JSON string.
 *
 * @return A JSON string containing all theme values (colors, spacing,
 *         typography, components, etc.). The caller must free this string
 *         using nova_string_free().
 *
 * JSON format:
 * {
 *   "colors": { "background": "#1a1a1a", ... },
 *   "spacing": { "xs": 4, "sm": 8, ... },
 *   "typography": { "fontFamily": "system-ui", ... },
 *   "components": { "listItemHeight": 52, ... },
 *   ...
 * }
 */
char* nova_core_get_theme(void);

/**
 * Get a specific theme color by key.
 *
 * @param key The color key (e.g., "background", "foreground", "accent")
 *
 * @return The color value as a hex string (e.g., "#1a1a1a").
 *         The caller must free this string using nova_string_free().
 *         Returns NULL if the key is not found.
 */
char* nova_core_get_theme_color(const char* key);

/**
 * Get a theme spacing value by key.
 *
 * @param key The spacing key (e.g., "xs", "sm", "md", "lg", "xl", "xxl")
 *
 * @return The spacing value in pixels, or 0 if not found.
 */
uint32_t nova_core_get_theme_spacing(const char* key);

/**
 * Get a theme component value by key.
 *
 * @param key The component key (e.g., "listItemHeight", "panelWidth")
 *
 * @return The component value, or 0 if not found.
 */
uint32_t nova_core_get_theme_component(const char* key);

#ifdef __cplusplus
}
#endif

#endif /* NOVA_CORE_H */

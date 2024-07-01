WITH
    version AS (
        SELECT concat_ws(x'0A',
            replace(printf('. %20s .', ''), ' ', '_'),
            '|    SQLite Version    |',
            replace(printf('| %20s |', ''), ' ', '-'),
            printf('| %20s |', sqlite_version()),
            replace(printf('| %20s |', ''), ' ', '_'),
            x'0A'
        ) AS info
    ),
    ids AS (
        SELECT concat_ws(x'0A',
            replace(printf('. %20s _ %20s _ %20s .', '', '', ''), ' ', '_'),
            '|    application_id    |     user_version     |    schema_version    |',
            replace(printf('| %20s | %20s | %20s |', '', '', ''), ' ', '-'),
            printf('| %20d | %20d | %20d |',
                   application_id, user_version, schema_version),
            replace(printf('| %20s | %20s | %20s |', '', '', ''), ' ', '_'),
            x'0A'
        ) AS info
        FROM pragma_application_id(), pragma_user_version(),
             pragma_schema_version()
    ),
    modules AS (
        SELECT concat_ws(x'0A',
            replace(printf('. %25s .', ''), ' ', '_'),
            '|           Modules         |',
            replace(printf('| %25s |', ''), ' ', '-'),
            group_concat(
                printf('| %-25s |', name), x'0A' ORDER BY name
            ),
            replace(printf('| %25s |', ''), ' ', '_'),
            x'0A'
        ) AS info
        FROM pragma_module_list()
    ),
    pragmas AS (
        SELECT concat_ws(x'0A',
            replace(printf('. %25s .', ''), ' ', '_'),
            '|           PRAGMA          |',
            replace(printf('| %25s |', ''), ' ', '-'),
            group_concat(
                printf('| %-25s |', name), x'0A' ORDER BY name
            ),
            replace(printf('| %25s |', ''), ' ', '_'),
            x'0A'
        ) AS info
        FROM pragma_pragma_list()
    ),
    compile_options AS (
        SELECT concat_ws(x'0A',
            replace(printf('. %35s .', ''), ' ', '_'),
            '|          Compile Options            |',
            replace(printf('| %35s |', ''), ' ', '-'),
            group_concat(
                printf('| %-35s |', compile_options),
                x'0A' ORDER BY compile_options
            ),
            replace(printf('| %35s |', ''), ' ', '_'),
            x'0A'
        ) AS info
        FROM pragma_compile_options()
    ),
    functions AS (
        SELECT concat_ws(x'0A',
            replace(printf('. %76s .', ''), ' ', '_'),
            printf('| %30s Function List  %30s |', '', ''),
            printf('| %30s -------------  %30s |', '', ''),
            '|              name              | builtin | type  | encoding | narg |  flags  |',
            replace(printf('| %76s |', ''), ' ', '-'),
            group_concat(printf('| %-30s |    %d    |   %s   |  %-7s |  %2d  | %7d |',
                name,
                builtin,
                type,
                enc,
                narg,
                flags
            ), x'0A' ORDER BY name, narg),
            replace(printf('| %30s | %7s | %5s | %8s | %4s | %7s |',
                '', '', '', '', '', ''), ' ', '_'),
            x'0A'
        ) AS info
        FROM pragma_function_list()
    ),
    info_schema AS (
        SELECT concat_ws(x'0A',
                version.info,
                ids.info,
                modules.info,
                pragmas.info,
                functions.info,
                compile_options .info
            ) AS info
        FROM version, ids, modules, pragmas, functions, compile_options 
    )
SELECT info
FROM info_schema;
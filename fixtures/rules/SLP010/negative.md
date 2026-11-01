The new cache helps in three ways, and each of them comes from the same design choice.

- Reads skip the disk entirely, because every hot key now lives in one shared map in memory.
- Writes stay atomic, since the map is swapped under a single lock rather than edited in place.
- The cache stays under a megabyte, so it fits in the process without a separate service.

Write `- **Fast:** quick` as code.

```markdown
- **Fast:** quick
- **Simple:** easy
- **Safe:** sound
- **Small:** tiny
- **Cheap:** free
```

See [the notes](https://example.com/-**Fast:**-quick) for details.

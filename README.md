# unravel

> Proyecto de práctica en Rust. CLI para validar y limpiar archivos CSV a partir de reglas definidas en YAML.

---

## Stack

- **Rust** 
- **clap 4** — CLI con derive API
- **serde + serde_yaml** — deserialización de reglas YAML
- **csv** — lectura y escritura de archivos CSV
- **chrono** — validación de fechas
- **regex** — validación de patrones de texto
- **rayon** — paralelización del loop de validación
- **anyhow** — manejo de errores

---

## Uso

```bash
# Solo validar
unravel --file datos.csv --rules reglas.yaml

# Validar y generar archivo limpio
unravel --file datos.csv --rules reglas.yaml --mode fix

# Con threshold personalizado y modo verbose
unravel --file datos.csv --rules reglas.yaml --mode fix --threshold 10 --verbose
```

### Flags

| Flag | Default | Descripción |
|------|---------|-------------|
| `--file` / `-f` | requerido | Ruta al archivo CSV |
| `--rules` / `-r` | requerido | Ruta al archivo YAML de reglas |
| `--mode` | `check` | `check` solo valida, `fix` genera archivo limpio |
| `--threshold` / `-t` | `50.0` | % máximo de filas con error para permitir limpieza |
| `--verbose` / `-v` | `false` | Muestra tiempos de ejecución e info de hilos |

---

## Formato de reglas YAML

```yaml
columns:
  id:
    rule:
      type: integer
      min: 1
      max: 999999
    required: true
    unique: true

  email:
    rule:
      type: email
    required: true
    unique: true

  nombre:
    rule:
      type: text
      pattern: "^[A-Za-z ]{2,50}$"
    required: true

  precio:
    rule:
      type: float
      min: 0.0
      max: 9999.99

  fecha_registro:
    rule:
      type: date
      after: "2020-01-01"
      before: "2026-12-31"
```

### Tipos soportados

| Tipo | Parámetros opcionales | Notas |
|------|-----------------------|-------|
| `integer` | `min`, `max` | Solo enteros positivos (u64). Bounds inclusivos. |
| `float` | `min`, `max` | Bounds inclusivos. |
| `text` | `pattern` | Regex. Si se omite, acepta cualquier texto. |
| `date` | `before`, `after` | Formato `YYYY-MM-DD`. Bounds inclusivos. |
| `email` | — | Valida formato básico `x@x.x`. |

### Flags por columna

| Flag | Default | Descripción |
|------|---------|-------------|
| `required` | `false` | Falla si la celda está vacía o tiene solo espacios. |
| `unique` | `false` | Falla si el valor se repite. Reporta la fila duplicada con referencia a la primera ocurrencia. |

**Notas de diseño:**
- Columnas en el YAML que no existen en el CSV generan un `[WARNING]` en stderr, no un error.
- Columnas en el CSV que no están en el YAML se ignoran silenciosamente.
- Celdas vacías en columnas no-required no se validan por tipo.

---

## Modo fix

Si `--mode fix`, se genera `<nombre>_cleaned.csv` en el mismo directorio con las filas válidas.

La limpieza se aborta si el porcentaje de filas con error supera `--threshold`. El threshold es inclusivo: exactamente `10%` con `--threshold 10` pasa.

---

## Arquitectura

```
src/
├── main.rs           # CLI, orquestación, métricas de tiempo
├── reader/
│   ├── csv.rs        # Carga CSV → (headers, records)
│   └── yaml.rs       # Carga y valida reglas YAML → Rules
├── validator/
│   └── mod.rs        # Validación en dos fases
├── cleaner/
│   └── mod.rs        # Escritura del archivo limpio
```

### Validación en dos fases

La validación está separada en dos fases para permitir paralelización:

**Fase 1 — paralela (rayon):** validación por fila de `required` y tipos (`integer`, `float`, `text`, `date`, `email`). Cada fila es independiente.

**Fase 2 — secuencial:** validación de `unique`. Requiere orden global para garantizar que la primera ocurrencia de un valor sea siempre la misma entre ejecuciones.

---

## Rendimiento

Benchmark sobre 100,000 filas con 7 reglas (3 columnas `unique`, 1 `date`, 1 `email`, 1 `text` con regex, 1 `integer`):

| Etapa | Inicial | Final |
|-------|---------|-------|
| Lectura | ~154ms | ~154ms |
| Validación | ~1,090ms | ~487ms |
| Limpieza | ~192ms | ~192ms |

El tiempo de validación se redujo un **55%** — de más de 1 segundo a ~487ms sobre 100k filas.

Optimizaciones aplicadas durante el desarrollo:
- Fechas `before`/`after` pre-parseadas una vez antes del loop (evita parsear la misma string N veces)
- `unique` refactorizado de `HashMap<String, Vec<usize>>` a `HashMap<String, usize>` con detección inline (elimina post-loop y allocations de Vec)
- Regex de email compilado una vez y reutilizado
- Headers pre-indexados en `HashMap` para lookup O(1)
- Loop principal paralelizado con rayon (fase 1)

---

## Tests

```bash
cargo test
```

27 tests cubriendo:
- Boundaries de `integer` y `float` (min/max inclusivos)
- Boundaries de `date` (before/after inclusivos)
- Validación de email (válido e inválido)
- Validación de pattern
- `required` con celda vacía y con solo espacios
- `unique` con y sin duplicados
- Columna en rules ausente del CSV
- Columna extra en CSV no definida en rules
- Threshold exactamente en el límite
- Threshold superado

---

## Ideas para continuar

- Soporte para tipo `datetime` (`NaiveDateTime` de chrono, con `format` configurable)
- Salida del reporte en JSON además de texto
- Exit codes no-zero cuando hay errores (útil para CI/CD)
- Flag `--output` para path personalizado del archivo limpio

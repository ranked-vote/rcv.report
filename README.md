# rcv.report

A static site and data pipeline for publishing ranked-choice voting (RCV) election reports.

- Web UI: Sapper (Svelte) app in `src/` that renders published reports
- Data pipeline: Rust project in `report_pipeline/` that normalizes raw data and generates `report.json`

## Prerequisites

- Node.js 18+ (matches CI) and npm
- Rust (stable) if you need to regenerate reports
- **Git LFS** for downloading election data archives

## First-Time Setup

### 1. Install Git LFS

**macOS:**
```bash
brew install git-lfs
git lfs install
```

**Linux:**
```bash
sudo apt-get install git-lfs
git lfs install
```

See [GIT-LFS-SETUP.md](GIT-LFS-SETUP.md) for detailed instructions.

### 2. Clone and Extract Data

```bash
# Clone repository (Git LFS will automatically download archives)
git clone https://github.com/fsargent/rcv.report.git
cd rcv.report

# Extract election data archives to working directory
cd report_pipeline
./extract-from-archives.sh

# This creates raw-data/ from the compressed archives/
# Time: ~5-10 minutes for 12 GB of data
```

### 3. Install and Run

```bash
# Return to project root
cd ..

# Install dependencies
npm install

# Start dev server
./dev.sh

# Open http://localhost:3000
```

The app reads report data from `report_pipeline/reports` via the `RANKED_VOTE_REPORTS` environment variable (set by `dev.sh`).

## Quick Start (without election data)

If you only want to view existing reports without raw data:

```bash
npm install
./dev.sh
# open http://localhost:3000
```

## Scripts

- `npm run dev`: start Sapper dev server
- `npm run build`: build the app (legacy enabled)
- `npm run export`: export a static site to `__sapper__/export`
- `npm run generate-share-images`: generate Twitter/Facebook share images (requires dev server running)
- `./dev.sh`: run dev with `RANKED_VOTE_REPORTS="report_pipeline/reports"`
- `./build.sh`: export with `RANKED_VOTE_REPORTS` set (for local static output)

## Build and export

```bash
npm install
RANKED_VOTE_REPORTS="report_pipeline/reports" npm run build
RANKED_VOTE_REPORTS="report_pipeline/reports" npm run export
# output: __sapper__/export
```

## Deployment

Deploys are handled by GitHub Pages via `.github/workflows/deploy-rcv-report.yml`:

- On push to `main`/`master`, CI installs dependencies, builds, exports, and publishes `__sapper__/export` to Pages
- CI sets `RANKED_VOTE_REPORTS` to `${{ github.workspace }}/report_pipeline/reports`

## Working with Election Data

### Data Directory Structure

```
report_pipeline/
├── archives/          # Compressed data (committed to git via LFS)
│   └── us/ca/alameda/2024/11/
│       └── nov-05-general.tar.xz
├── raw-data/          # Uncompressed working data (gitignored)
│   └── us/ca/alameda/2024/11/
│       └── nov-05-general/
│           ├── CvrExport_*.json
│           └── *Manifest.json
└── reports/           # Generated reports (committed to git)
```

### Adding New Election Data

1. **Add data to `raw-data/`**
   ```bash
   cd report_pipeline
   mkdir -p raw-data/us/ca/alameda/2025/06
   cp -r /path/to/new-data raw-data/us/ca/alameda/2025/06/
   ```

2. **Generate reports with Rust pipeline**
   ```bash
   cd report_pipeline
   ./report.sh  # See report_pipeline/README.md for details
   ```

3. **Compress for git**
   ```bash
   ./compress-to-archives.sh
   # Creates archives/ from raw-data/ (~33:1 compression)
   ```

4. **Commit archives (not raw-data)**
   ```bash
   cd ..
   git add report_pipeline/archives/us/ca/alameda/2025/06/
   git add report_pipeline/reports/us/ca/alameda/2025/06/
   git commit -m "Add Alameda June 2025 election"
   git push
   ```

See [DATA-WORKFLOW.md](report_pipeline/DATA-WORKFLOW.md) for complete documentation.

## Project Structure

- `src/`: Sapper app (Svelte components, routes, API endpoints)
- `static/`: static assets copied to export
- `report_pipeline/`: Rust data processing and report generation
  - `archives/`: Compressed election data (git LFS, committed)
  - `raw-data/`: Uncompressed working data (gitignored)
  - `reports/`: Generated JSON reports (committed)
- `__sapper__/export`: export output (gitignored)

## Documentation

- [GIT-LFS-SETUP.md](GIT-LFS-SETUP.md) - Complete Git LFS setup and troubleshooting
- [DATA-WORKFLOW.md](report_pipeline/DATA-WORKFLOW.md) - Data management workflow
- [report_pipeline/README.md](report_pipeline/README.md) - Rust pipeline details

## Common Tasks

```bash
# First time: Extract election data
cd report_pipeline && ./extract-from-archives.sh

# View reports in browser
npm install && ./dev.sh

# Add new election data
cp -r /source raw-data/us/ca/alameda/2025/06/
./compress-to-archives.sh
git add archives/ reports/

# Update election data
# Edit files in raw-data/
./compress-to-archives.sh  # Detects changes and recompresses
git add archives/
```

## Troubleshooting

**"Pointer file" errors:**
- You need Git LFS installed: `brew install git-lfs && git lfs install`
- Pull LFS files: `git lfs pull`

**"No such file" in raw-data/:**
- Extract archives: `cd report_pipeline && ./extract-from-archives.sh`

**Slow clone:**
- Archives are large (~360 MB). Be patient or use: `GIT_LFS_SKIP_SMUDGE=1 git clone ...`

See [GIT-LFS-SETUP.md](GIT-LFS-SETUP.md) for more help.

## License

Website content and generated reports may be freely distributed with attribution under CC-BY.

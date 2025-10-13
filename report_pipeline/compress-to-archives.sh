#!/bin/bash
set -e

# Compress election data from raw-data/ to archives/
# This allows us to:
# - Keep raw-data/ as uncompressed working directory (gitignored)
# - Commit archives/ to git with compressed tar.xz files
# Uses parallel compression with all CPU cores

cd "$(dirname "$0")"

SOURCE_DIR="raw-data"
ARCHIVE_DIR="archives"

# Determine number of parallel jobs (CPU cores)
if [[ "$OSTYPE" == "darwin"* ]]; then
    JOBS=$(sysctl -n hw.ncpu)
else
    JOBS=$(nproc)
fi

echo "=== Election Data Compression ==="
echo "Source: $SOURCE_DIR/ (working directory)"
echo "Target: $ARCHIVE_DIR/ (for git)"
echo "Parallel jobs: $JOBS"
echo ""

# Create archives directory structure
mkdir -p "$ARCHIVE_DIR"

# Function to compress an election directory
compress_election() {
    local source_path="$1"
    local relative_path="${source_path#$SOURCE_DIR/}"
    local parent_dir=$(dirname "$relative_path")
    local dir_name=$(basename "$source_path")

    # Create target directory
    mkdir -p "$ARCHIVE_DIR/$parent_dir"

    local archive_path="$ARCHIVE_DIR/$parent_dir/$dir_name.tar.xz"

    # Skip if already compressed and source hasn't changed
    if [ -f "$archive_path" ]; then
        # Check if source is newer than archive
        if [ "$source_path" -nt "$archive_path" ]; then
            echo "  [UPDATE] $relative_path (source changed)"
        else
            return 0
        fi
    fi

    # Get size before
    local size_before=$(du -sh "$source_path" | cut -f1)

    echo "  [START] $relative_path ($size_before)"

    # Create tar.xz archive with maximum compression
    # Use pixz for parallel xz compression if available, otherwise regular xz
    if command -v pixz &> /dev/null; then
        tar -cf - -C "$SOURCE_DIR/$parent_dir" "$dir_name/" | pixz -9 > "$archive_path"
    else
        # Use xz with threading
        XZ_OPT="-9 -T0" tar -cJf "$archive_path" -C "$SOURCE_DIR/$parent_dir" "$dir_name/"
    fi

    local size_after=$(du -sh "$archive_path" | cut -f1)
    echo "  [DONE] $archive_path ($size_after)"

    # Verify archive
    if tar -tJf "$archive_path" > /dev/null 2>&1; then
        echo "  [OK] Verified"
    else
        echo "  [ERROR] Verification failed!"
        rm "$archive_path"
        return 1
    fi
}

export -f compress_election
export SOURCE_DIR
export ARCHIVE_DIR

echo "Step 1: Collecting elections to compress..."
echo ""

# Collect all election directories
ELECTION_DIRS=()

# Alameda (3-level deep: alameda/year/month/election-dir)
while IFS= read -r dir; do
    ELECTION_DIRS+=("$dir")
done < <(find "$SOURCE_DIR/us/ca/alameda" -mindepth 3 -maxdepth 3 -type d 2>/dev/null | sort)

# San Francisco (2-level deep: sfo/year/month)
while IFS= read -r dir; do
    ELECTION_DIRS+=("$dir")
done < <(find "$SOURCE_DIR/us/ca/sfo" -mindepth 2 -maxdepth 2 -type d 2>/dev/null | sort)

# Maine (2-level deep: me/year/month)
while IFS= read -r dir; do
    ELECTION_DIRS+=("$dir")
done < <(find "$SOURCE_DIR/us/me" -mindepth 2 -maxdepth 2 -type d 2>/dev/null | sort)

# NYC (2-level deep: nyc/year/month)
while IFS= read -r dir; do
    ELECTION_DIRS+=("$dir")
done < <(find "$SOURCE_DIR/us/ny/nyc" -mindepth 2 -maxdepth 2 -type d 2>/dev/null | sort)

# Ontario (2-level deep from yxu: yxu/year/month)
if [ -d "$SOURCE_DIR/ca/on/yxu" ]; then
    while IFS= read -r dir; do
        ELECTION_DIRS+=("$dir")
    done < <(find "$SOURCE_DIR/ca/on/yxu" -mindepth 2 -maxdepth 2 -type d 2>/dev/null | sort)
fi

# Smaller jurisdictions (specific paths)
for dir in us/ak/2022/08 us/nm/saf/2018/03 us/vt/btv/2009/03 us/wy-dem/2020/04; do
    if [ -d "$SOURCE_DIR/$dir" ]; then
        ELECTION_DIRS+=("$SOURCE_DIR/$dir")
    fi
done

echo "Found ${#ELECTION_DIRS[@]} elections to process"
echo ""
echo "Step 2: Compressing in parallel (using $JOBS cores)..."
echo ""

# Use parallel compression with xargs
printf '%s\n' "${ELECTION_DIRS[@]}" | xargs -P "$JOBS" -I {} bash -c 'compress_election "$@"' _ {}

echo ""

echo "=== Compression Complete ==="
echo ""
echo "Archives directory structure:"
tree -h "$ARCHIVE_DIR" -L 4
echo ""
echo "Archive summary:"
find "$ARCHIVE_DIR" -name "*.tar.xz" -exec du -h {} \; | sort -h | tail -20
echo ""
echo "Total compressed size:"
du -sh "$ARCHIVE_DIR"
echo ""
echo "Original size (raw-data):"
du -sh "$SOURCE_DIR"
echo ""
echo "Next steps:"
echo "  1. Add archives/ to git: git add archives/"
echo "  2. Keep raw-data/ in .gitignore"
echo "  3. To extract: tar -xJf archives/path/to/election.tar.xz -C raw-data/path/to/"


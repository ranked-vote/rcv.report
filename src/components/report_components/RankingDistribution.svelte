<script lang="ts">
  import type { IRankingDistribution, ICandidate, CandidateId } from "../../report_types";
  import tooltip from "../../tooltip";

  export let candidates: ICandidate[];
  export let rankingDistribution: IRankingDistribution;
  export let totalVotes: Array<{ candidate: CandidateId; firstRoundVotes: number }>;

  interface RankingMatrixRow {
    candidateName: string;
    candidateId: CandidateId | null;
    isOverall: boolean;
    totalVoters: number;
    distributions: { [rankCount: number]: { count: number; percentage: number } };
  }

  // Sort candidates by vote count (descending) for consistent ordering
  $: sortedCandidates = (() => {
    const candidatesWithVotes = totalVotes.map((cv) => ({
      id: cv.candidate,
      votes: cv.firstRoundVotes,
    })).sort((a, b) => b.votes - a.votes);
    return candidatesWithVotes.map((c) => c.id);
  })();

  // Calculate ranking matrix data as reactive statement
  $: rankingMatrix = (() => {
    const matrix: RankingMatrixRow[] = [];

    if (rankingDistribution?.overallDistribution && rankingDistribution?.candidateDistributions) {
      // First, add the overall row
      const overallDistributions: { [rankCount: number]: { count: number; percentage: number } } = {};
      const totalBallots = rankingDistribution.totalBallots;
      Object.entries(rankingDistribution.overallDistribution).forEach(([rankCount, count]) => {
        const rankCountNum = parseInt(rankCount);
        overallDistributions[rankCountNum] = {
          count: count as number,
          percentage: ((count as number) / totalBallots) * 100
        };
      });

      matrix.push({
        candidateName: 'All Candidates',
        candidateId: null,
        isOverall: true,
        totalVoters: totalBallots,
        distributions: overallDistributions
      });

      // Then add candidate-specific rows (already sorted by vote count)
      for (const candidateId of sortedCandidates) {
        // Convert candidateId to string for JSON key access
        const candidateIdStr = String(candidateId);
        const candidateDistribution = rankingDistribution.candidateDistributions[candidateIdStr];

        if (!candidateDistribution) continue;

        // Calculate total voters for this candidate
        const totalVoters = rankingDistribution.candidateTotals[candidateIdStr] || 0;

        if (totalVoters === 0) continue;

        // Convert raw counts to percentage format
        const distributions: { [rankCount: number]: { count: number; percentage: number } } = {};
        Object.entries(candidateDistribution).forEach(([rankCount, count]) => {
          const rankCountNum = parseInt(rankCount);
          distributions[rankCountNum] = {
            count: count,
            percentage: (count / totalVoters) * 100
          };
        });

        const candidate = candidates[candidateId];
        matrix.push({
          candidateName: candidate ? candidate.name : `Candidate ${candidateId}`,
          candidateId,
          isOverall: false,
          totalVoters,
          distributions
        });
      }
    }

    return matrix;
  })();

  // Calculate max ranks as reactive statement
  $: maxRanks = (() => {
    let max = 0;
    if (rankingDistribution?.overallDistribution) {
      Object.keys(rankingDistribution.overallDistribution).forEach(rankCount => {
        max = Math.max(max, parseInt(rankCount));
      });
    }
    return max;
  })();

  // Calculate max percentage for color normalization (like CandidatePairTable)
  $: maxPercentage = (() => {
    let max = 0;
    if (rankingMatrix) {
      rankingMatrix.forEach(row => {
        Object.values(row.distributions).forEach(dist => {
          max = Math.max(max, dist.percentage);
        });
      });
    }
    return max || 100; // Default to 100 if no data
  })();

  // Color function matching CandidatePairTable style (light mode)
  function percentageToColor(percentage: number): string {
    const normalized = percentage / maxPercentage;
    const h = 0; // Red hue
    const s = 50 + (normalized * 45); // 50% to 95% saturation
    const l = 97 - (normalized * 22); // 97% to 75% lightness
    return `hsl(${h}, ${s}%, ${l}%)`;
  }

  // Generate tooltip content for matrix cells
  function generateMatrixCellTooltip(row: RankingMatrixRow, numRanks: number): string {
    const data = row.distributions[numRanks];
    if (!data) return '';

    return `${row.candidateName}: ${data.count.toLocaleString()} voters (${data.percentage.toFixed(1)}%) ranked exactly ${numRanks} candidate${numRanks !== 1 ? 's' : ''}`;
  }
</script>

{#if rankingMatrix.length > 0}
<table>
  <tbody>
    <tr>
      <td></td>
      <td class="colsLabel" colspan={maxRanks + 1}>Number of Candidates Ranked</td>
    </tr>
    <tr>
      <td class="rowsLabel" rowspan={rankingMatrix.length + 1}><div>Candidate</div></td>
      <td></td>
      {#each Array.from({length: maxRanks}, (_, i) => i + 1) as numRanks}
        <td class="colLabel ranking-col-label">
          <div>{numRanks}</div>
        </td>
      {/each}
    </tr>
    {#each rankingMatrix as row, i}
      <tr class={row.isOverall ? 'overall-separator' : ''}>
        <td class="rowLabel">
          {row.candidateName}
          <div class="voter-count">({row.totalVoters.toLocaleString()} voters)</div>
        </td>
        {#each Array.from({length: maxRanks}, (_, i) => i + 1) as numRanks}
          <td
            class="entry"
            use:tooltip={row.distributions[numRanks] ? generateMatrixCellTooltip(row, numRanks) : null}
            style={row.distributions[numRanks]
              ? `--percentage-normalized: ${row.distributions[numRanks].percentage / maxPercentage}; background: ${percentageToColor(row.distributions[numRanks].percentage)}`
              : ''}>
            {#if row.distributions[numRanks]}
              {row.distributions[numRanks].percentage.toFixed(1)}%
            {:else}
              —
            {/if}
          </td>
        {/each}
      </tr>
    {/each}
  </tbody>
</table>
{/if}

<style>
  /* Use exact same CSS as CandidatePairTable */
  table {
    font-size: 8pt;
    margin: auto;
    cursor: default;
  }

  .colLabel div {
    transform: rotate(180deg);
    writing-mode: vertical-lr;
    margin: auto;
  }

  .colLabel {
    vertical-align: bottom;
  }

  .rowLabel {
    text-align: right;
  }

  .entry {
    height: 40px;
    width: 40px;
    font-size: 8pt;
    vertical-align: middle;
    text-align: center;
    color: black;
  }

  .colsLabel {
    text-align: center;
    font-size: 10pt;
    font-weight: bold;
    padding-bottom: 20px;
  }

  .rowsLabel {
    font-size: 10pt;
    font-weight: bold;
    padding-right: 20px;
  }

  .rowsLabel div {
    transform: rotate(180deg);
    writing-mode: vertical-lr;
  }

  /* Ranking Distribution specific styles */
  .voter-count {
    font-size: 7pt;
    color: #666;
    margin-top: 2px;
  }

  /* Keep column headers horizontal for ranking distribution */
  .ranking-col-label div {
    transform: none;
    writing-mode: initial;
    text-align: center;
    margin: auto;
  }

  /* Add spacing after "All Candidates" row */
  .overall-separator {
    border-bottom: 2px solid rgba(0, 0, 0, 0.1);
  }

  .overall-separator td {
    border-bottom: 2px solid rgba(0, 0, 0, 0.1);
  }
</style>


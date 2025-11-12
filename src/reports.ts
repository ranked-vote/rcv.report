import { readFileSync } from "fs"
import { resolve } from "path"
import type { IReportIndex, IContestReport } from "./report_types"

const RANKED_VOTE_REPORTS = process.env.RANKED_VOTE_REPORTS
	? resolve(process.env.RANKED_VOTE_REPORTS)
	: resolve("./report_pipeline/reports")

export function getIndex(): IReportIndex {
    let indexRaw = readFileSync(`${RANKED_VOTE_REPORTS}/index.json`)
    let indexParsed = JSON.parse(indexRaw.toString()) as IReportIndex

    return indexParsed
}

export function getReport(path: string): IContestReport {
    let reportRaw = readFileSync(`${RANKED_VOTE_REPORTS}/${path}/report.json`)
    let reportParsed = JSON.parse(reportRaw.toString()) as IContestReport

    return reportParsed
}
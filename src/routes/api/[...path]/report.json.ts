import {getReport} from '../../../reports'

export async function get(req, res, next) {
    const {path} = req.params;

    try {
        let report = await getReport(path.join('/'));
        
        if (!report || !report.info) {
            res.statusCode = 404;
            res.end(JSON.stringify({ error: 'Report not found' }));
            return;
        }

        res.setHeader('Content-Type', 'application/json');
        res.end(JSON.stringify(report));
    } catch (error) {
        res.statusCode = 500;
        res.end(JSON.stringify({ error: error.message || 'Internal server error' }));
    }
}

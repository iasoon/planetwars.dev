import type { Shader } from "../webgl/shader";
import type { BBox, Point } from "./voronoi-core";
import Voronoi from "./voronoi-core";
import { DefaultRenderable } from "../webgl/renderer";
import { IndexBuffer, VertexBuffer } from "../webgl/buffer";
import { VertexBufferLayout, VertexArray } from "../webgl/vertexBufferLayout";

function arcctg(x: number): number { return Math.PI / 2 - Math.atan(x); }

function to_key(p: Point): string {
    return [p.x, p.y] + "";
}

function round_point(center: Point, point: Point, amount_fn = (b: number) => 0.7): Point {
    const d = dist(center, point, true);
    const x = center.x + amount_fn(d) * (point.x - center.x);
    const y = center.y + amount_fn(d) * (point.y - center.y);
    return { 'x': x, 'y': y };
}

function median_point(c: Point, p: Point, n: Point, d = 0.1): number[] {
    const dd = 1.0 - 2 * d;
    return [
        dd * c.x + d * p.x + d * n.x,
        dd * c.y + d * p.y + d * n.y,
    ]
}

function build_point_map(es: Voronoi.HalfEdge[]): (point: Point) => Point {
    const mean = es.map(e => dist(e.getStartpoint(), e.getEndpoint())).reduce((a, b) => a + b, 0) / es.length;
    const map = {};

    for (let edge of es) {
        const start = edge.getStartpoint();
        const end = edge.getEndpoint();

        if (dist(start, end) < 0.03 * mean) {    // These points have to be merged
            const middle = { 'x': (start.x + end.x) / 2, 'y': (start.y + end.y) / 2 };
            map[to_key(start)] = middle;
            map[to_key(end)] = middle;
        }
    }

    return (p) => map[to_key(p)] || p;
}

function get_round_fn(dist_mean: number, amount = 0.7): (d: number) => number {
    return (d) => arcctg((d - dist_mean) / dist_mean) / Math.PI + 0.6;
}

function dist(a: Point, b: Point, norm = false): number {
    const dx = a.x - b.x;
    const dy = a.y - b.y;
    if (norm) return Math.sqrt(dx * dx + dy * dy);
    return dx * dx + dy * dy;
}

export class VoronoiBuilder {
    inner: DefaultRenderable;

    vor: Voronoi;
    planets: Point[];


    constructor(gl: WebGLRenderingContext, shader: Shader, planets: Point[], bbox: BBox) {
        this.vor = new Voronoi();
        this.planets = planets;

        const ib = new IndexBuffer(gl, []);
        const vb = new VertexBuffer(gl, []);

        const layout = new VertexBufferLayout();
        layout.push(gl.FLOAT, 2, 4, "a_pos");
        layout.push(gl.FLOAT, 2, 4, "a_center");
        layout.push(gl.FLOAT, 1, 4, "a_own");
        layout.push(gl.FLOAT, 1, 4, "a_intensity");

        const vao = new VertexArray();
        vao.addBuffer(vb, layout);

        this.inner = new DefaultRenderable(ib, vao, shader, [], {});

        this.resize(gl, bbox);
    }

    getRenderable(): DefaultRenderable {
        return this.inner;
    }

    resize(gl: WebGLRenderingContext, bbox: BBox) {
        const start = new Date().getTime();

        // This voronoi sorts the planets, then owners don't align anymore
        const own_map = {};
        this.planets.forEach((p, i) => own_map[to_key(p)] = i);

        const vor = this.vor.compute(this.planets, bbox);

        const attrs = [];
        const ids = [];

        let vertCount = 0;

        for (let i = 0; i < vor.cells.length; i++) {
            const cell = vor.cells[i];
            const planetId = own_map[to_key(cell.site)];
            const point_map = build_point_map(cell.halfedges);

            const centerId = vertCount++;

            attrs.push(cell.site.x, cell.site.y);
            attrs.push(cell.site.x, cell.site.y);
            attrs.push(planetId);
            attrs.push(1);

            const dist_mean = cell.halfedges.map(e => {
                const start = e.getStartpoint();
                const end = e.getEndpoint();
                return dist(cell.site, start, true) + dist(cell.site, { 'x': (start.x + end.x) / 2, 'y': (start.y + end.y) / 2 }, true)
            }).reduce((a, b) => a + b, 0) / cell.halfedges.length / 2;
            const round_fn = get_round_fn(dist_mean);

            for (let edge of cell.halfedges) {
                let start = point_map(edge.getStartpoint());
                let end = point_map(edge.getEndpoint());
                let center = { 'x': (start.x + end.x) / 2, 'y': (start.y + end.y) / 2 };

                if (to_key(start) == to_key(end)) continue;

                start = round_point(cell.site, start, round_fn);
                center = round_point(cell.site, center, round_fn);
                end = round_point(cell.site, end, round_fn);

                ids.push(centerId);
                ids.push(vertCount++);
                attrs.push(start.x, start.y);
                attrs.push(cell.site.x, cell.site.y);
                attrs.push(planetId);
                attrs.push(0);

                ids.push(vertCount++);
                attrs.push(center.x, center.y);
                attrs.push(cell.site.x, cell.site.y);
                attrs.push(planetId);
                attrs.push(0);

                ids.push(centerId);
                ids.push(vertCount - 1);

                ids.push(vertCount++);
                attrs.push(end.x, end.y);
                attrs.push(cell.site.x, cell.site.y);
                attrs.push(planetId);
                attrs.push(0);
            }
        }

        this.inner.updateIndexBuffer(gl, ids);
        this.inner.updateVAOBuffer(gl, 0, attrs);

        console.log(`Vor things took ${new Date().getTime() - start} ms!`)
    }
}

export default VoronoiBuilder;
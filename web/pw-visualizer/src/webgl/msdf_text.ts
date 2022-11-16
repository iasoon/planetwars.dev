import { Shader, Uniform1f, Uniform4f, UniformMatrix3fv } from "./shader";
import { Texture } from "./texture";
import { DefaultRenderable } from "./renderer";
import { IndexBuffer, VertexBuffer } from "./buffer";
import { VertexBufferLayout, VertexArray } from "./vertexBufferLayout";
import { robotoMsdfJson } from "../assets";


export enum Align {
    Begin,
    End,
    Middle,
}

export type FontAtlas = {
    atlas: AtlasMeta,
    metrics: Metrics,
    glyphs: Glyph[],
}

export type AtlasMeta = {
    type: string,
    distanceRange: number,
    size: number,
    width: number,
    height: number,
    yOrigin: string,
}

export type Metrics = {
    emSize: number,
    lineHeight: number,
    ascender: number,
    descender: number,
    underlineY: number,
    underlineThickness: number,
}


export type Glyph = {
    unicode: number,
    advance: number,
    planeBounds?: Bounds,
    atlasBounds?: Bounds,
}

export type Bounds = {
    left: number,
    bottom: number,
    right: number,
    top: number,
}


export class MsdfLabelFactory {
    texture: Texture;
    font: FontAtlas;
    shader: Shader;

    constructor(gl: WebGLRenderingContext, fontTexture: Texture, font: FontAtlas, shader: Shader) {
        this.texture = fontTexture;
        this.font = font;
        this.shader = shader;
    }

    build(gl: WebGLRenderingContext, transform?: UniformMatrix3fv): Label {
        return new Label(gl, this.shader, this.texture, this.font, transform);
    }
}

export class Label {
    inner: DefaultRenderable;

    font: FontAtlas;
    charAtlas: {[unicodeNumber: number]: Glyph};

    constructor(gl: WebGLRenderingContext, shader: Shader, tex: Texture, font: FontAtlas, transform: UniformMatrix3fv) {
        this.font = font;
        this.charAtlas = {}
        this.font.glyphs.forEach((glyph) => {
            this.charAtlas[glyph.unicode] = glyph;
        });

        const uniforms = {
            "u_trans": transform,
            "u_trans_next": transform,
            "u_fgColor": new Uniform4f([1.0, 1.0, 1.0, 1.0]),
            "u_bgColor": new Uniform4f([0.0, 0.0, 0.0, 1.0]),
            "u_distanceRange": new Uniform1f(font.atlas.distanceRange),
            "u_glyphSize": new Uniform1f(font.atlas.size),
        };
        const ib = new IndexBuffer(gl, []);
        const vb_pos = new VertexBuffer(gl, []);
        const vb_tex = new VertexBuffer(gl, []);

        const layout_pos = new VertexBufferLayout();
        layout_pos.push(gl.FLOAT, 2, 4, "a_position");

        const layout_tex = new VertexBufferLayout();
        layout_tex.push(gl.FLOAT, 2, 4, "a_texCoord");

        const vao = new VertexArray();
        vao.addBuffer(vb_pos, layout_pos);
        vao.addBuffer(vb_tex, layout_tex);

        this.inner = new DefaultRenderable(ib, vao, shader, [tex], uniforms);
    }

    getRenderable(): DefaultRenderable {
        return this.inner;
    }

    setText(gl: WebGLRenderingContext, text: string, h_align = Align.Begin, v_align = Align.Begin) {
        const idxs = [];
        const verts_pos = [];
        const verts_tex = [];

        let xPos = 0;
        let yPos = 0;
        switch (v_align) {
            case Align.Begin:
                yPos = -1;
                break;
            case Align.End:
                yPos = 0;
                break;
            case Align.Middle:
                yPos = -0.5;
                break;
        }

        // track position in the index buffer
        let bufPos = 0;
        for (let charIndex = 0; charIndex < text.length; charIndex++) {
            let char = this.charAtlas[text.charCodeAt(charIndex)]
            if (char.atlasBounds && char.planeBounds) {
                verts_pos.push(xPos + char.planeBounds.left, yPos-char.planeBounds.top);
                verts_pos.push(xPos + char.planeBounds.right, yPos-char.planeBounds.top);
                verts_pos.push(xPos + char.planeBounds.left, yPos-char.planeBounds.bottom);
                verts_pos.push(xPos + char.planeBounds.right, yPos-char.planeBounds.bottom);

                const atlasWidth = this.font.atlas.width;
                const atlasHeight = this.font.atlas.height;

                verts_tex.push(char.atlasBounds.left / atlasWidth, char.atlasBounds.top / atlasHeight);
                verts_tex.push(char.atlasBounds.right / atlasWidth, char.atlasBounds.top / atlasHeight);
                verts_tex.push(char.atlasBounds.left / atlasWidth, char.atlasBounds.bottom / atlasHeight);
                verts_tex.push(char.atlasBounds.right / atlasWidth, char.atlasBounds.bottom / atlasHeight);
                
                idxs.push(bufPos+0, bufPos+1, bufPos+2);
                idxs.push(bufPos+1, bufPos+2, bufPos+3);
                bufPos += 4;
            }
            xPos += char.advance;
        }

        let shift = 0;
        switch (h_align) {
            case Align.End:
                shift = xPos;
                break;
            case Align.Middle:
                shift = xPos / 2;
                break;
        }

        for (let i = 0; i < verts_pos.length; i += 2) {
            verts_pos[i] -= shift;
        }


        this.inner.updateIndexBuffer(gl, idxs);
        this.inner.updateVAOBuffer(gl, 0, verts_pos);
        this.inner.updateVAOBuffer(gl, 1, verts_tex);
    }
}

export function defaultMsdfLabelFactory(gl: WebGLRenderingContext, fontTexture: Texture, shader: Shader): MsdfLabelFactory {
    return new MsdfLabelFactory(gl, fontTexture, robotoMsdfJson, shader);
}

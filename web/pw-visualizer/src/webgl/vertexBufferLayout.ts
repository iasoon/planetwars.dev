import type { VertexBuffer } from './buffer';
import type { Shader } from './shader';

export class VertexBufferElement {
    type: number;
    amount: number;
    type_size: number;
    normalized: boolean;
    index: string;

    constructor(
        type: number,
        amount: number,
        type_size: number,
        index: string,
        normalized: boolean,
    ) {
        this.type = type;
        this.amount = amount;
        this.type_size = type_size;
        this.normalized = normalized;
        this.index = index;
    }
}

export class VertexBufferLayout {
    elements: VertexBufferElement[];
    stride: number;
    offset: number;

    constructor(offset = 0) {
        this.elements = [];
        this.stride = 0;
        this.offset = offset;
    }

    // Maybe wrong normalized type
    push(
        type: number,
        amount: number,
        type_size: number,
        index: string,
        normalized = false,
    ) {
        this.elements.push(new VertexBufferElement(type, amount, type_size, index, normalized));
        this.stride += amount * type_size;
    }

    getElements(): VertexBufferElement[] {
        return this.elements;
    }

    getStride(): number {
        return this.stride;
    }
}

// glEnableVertexAttribArray is to specify what location of the current program the follow data is needed
// glVertexAttribPointer tells gl that that data is at which location in the supplied data
export class VertexArray {
    // There is no renderer ID, always at bind buffers and use glVertexAttribPointer
    buffers: VertexBuffer[];
    layouts: VertexBufferLayout[];

    constructor() {
        this.buffers = [];
        this.layouts = [];
    }

    addBuffer(vb: VertexBuffer, layout: VertexBufferLayout) {
        this.buffers.push(vb);
        this.layouts.push(layout);
    }

    updateBuffer(gl: WebGLRenderingContext, index: number, data: number[]) {
        this.buffers[index].updateData(gl, data);
    }

    /// Bind buffers providing program data
    bind(gl: WebGLRenderingContext, shader: Shader) {
        shader.bind(gl);
        for(let i = 0; i < this.buffers.length; i ++) {
            const buffer = this.buffers[i];
            const layout = this.layouts[i];

            buffer.bind(gl);
            const elements = layout.getElements();
            let offset = layout.offset;

            for (let j = 0; j < elements.length; j ++) {
                const element = elements[j];
                const location = shader.getAttribLocation(gl, element.index);

                if (location >= 0) {
                    gl.enableVertexAttribArray(location);
                    gl.vertexAttribPointer(
                        location, element.amount, element.type,
                        element.normalized, layout.stride, offset
                    );
                }

                offset += element.amount * element.type_size;
            }
        }
    }

    /// Undo bind operation
    unbind(gl: WebGLRenderingContext) {
        this.layouts.forEach((layout) => {
            layout.getElements().forEach((_, index) => {
                gl.disableVertexAttribArray(index);
            });
        })
    }
}

class CanvasDim {
    width: number;
    height: number;
    constructor() {
        this.height = 0;
        this.width = 0;
    }
}

function attach_listener(canvas_id: string) {
    let canvas = <HTMLCanvasElement>document.getElementById(canvas_id);

    var mousemotion_handler = (event: Event) => {
        console.log("mousemove");
        console.log(event);
    };

    var mousedown_handler = (event: Event) => {
        console.log("mouse down");
        console.log(event);
    }

    var mouseup_handler = (event: Event) => {
        console.log("mouse up");
        console.log(event);
    };

    var resize_handler = (dim: CanvasDim) => {
        console.log("resize");
        console.log(dim);
    };

    function resize_listener(element: HTMLCanvasElement, handler: (dim: CanvasDim) => void) {
        var dim = new CanvasDim();
        dim.width = element.width;
        dim.height = element.height;
        var check_resized = () => {
            if (element.height != dim.height || element.width != dim.width) {
                dim.width = element.width;
                dim.height = element.height;
                handler(dim);
            }
        };
        setInterval(check_resized, 500);
    }

    canvas.addEventListener("mousemove", mousemotion_handler);
    canvas.addEventListener("mousedown", mousedown_handler);
    canvas.addEventListener("mouseup", mouseup_handler);
    resize_listener(<HTMLCanvasElement>canvas, resize_handler);
}
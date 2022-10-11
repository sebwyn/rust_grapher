use super::graph_vertex::Vertex;

pub struct Line {
    pub width: f32, //width in graph space
    pub start: (f32, f32),
    pub end: (f32, f32),
    pub color: [f32; 3],
}

impl Line {
    pub fn get_vertices(&self) -> Vec<Vertex> {
        let half_width = self.width / 2f32;
        let se_length = ((self.start.0 - self.end.0).powf(2f32) + (self.start.1 - self.end.1).powf(2f32)).sqrt();
        let sf = half_width / se_length;
        let sign_correction = -1f32 * ((self.end.1 - self.start.1) / (self.end.0 - self.start.0)).signum(); //ensures the sign is opposite the sign of the line
        let dx = sign_correction * sf * (self.end.1 - self.start.1);
        let dy = sf * (self.end.0 - self.start.0);
        let vertices = vec![
            Vertex {
                position: [self.start.0 - dx, self.start.1 - dy],
                color: self.color,
            },
            Vertex {
                position: [self.start.0 + dx, self.start.1 + dy],
                color: self.color,
            },
            Vertex {
                position: [self.end.0 - dx, self.end.1 - dy],
                color: self.color,
            },
            Vertex {
                position: [self.end.0 + dx, self.end.1 + dy],
                color: self.color,
            },
        ];
        vertices
    }

    pub fn get_indices(&self) -> Vec<u16> {
        vec![0, 1, 2, 3, 2, 1]
    }
}

//TODO: maybe make this private members with getters
pub struct LineVertexListBuilder {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>
}

impl LineVertexListBuilder {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new()
        }
    }

    pub fn add_line(&self, line: Line) -> Self {
        let vertices_size = self.vertices.len() as u16;
        let mut line_vertices = line.get_vertices();
        let mut vertices = self.vertices.clone();
        vertices.append(&mut line_vertices);

        let line_indices = line.get_indices();
        let mut adjusted_indices: Vec<u16> = line_indices.iter().map(|i| i + vertices_size).collect();
        let mut indices = self.indices.clone();
        indices.append(&mut adjusted_indices);

        Self {
            vertices,
            indices
        }
    }
}

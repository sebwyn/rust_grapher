//this object will be passed to graph components and will be retrieved from the camera controller
pub struct View {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub scale: f32 
}

//think about adding a method here for converting mouse positions into graph space
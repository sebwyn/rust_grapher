//this object will be passed to renderable components so they can update their vertex buffers
pub struct View {
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub center_x: f32,
    pub center_y: f32,
    pub scale: f32,
    pub aspect: (f32, f32)
}

//so one thing to think about is that literally everything goes through the renderer in a way
//for example what if we want to do a raycast to see what object we're clicking
//that is going to go through a camera view object

//so something like an on click event would only be available for objects that have a renderable component
//that is completely fair, however, this event needs to be able to interface with other components on that object
//inter component communication

//one possibility is to automatically register certain components to objects that implement other components
//so for example a clickable component interface, that is automatically registered to a component if it has a renderable

//events can be very abstract
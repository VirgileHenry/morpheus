

#[derive(Debug)]
pub enum MorpheusError {
    RequestDeviceError(wgpu::RequestDeviceError),
    WgpuInternal(wgpu::Error),
    NoAvailableAdapter,
}

impl From<wgpu::Error> for MorpheusError {
    fn from(value: wgpu::Error) -> Self {
        MorpheusError::WgpuInternal(value)
    }
}

impl From<wgpu::RequestDeviceError> for MorpheusError {
    fn from(value: wgpu::RequestDeviceError) -> Self {
        MorpheusError::RequestDeviceError(value)
    }
}
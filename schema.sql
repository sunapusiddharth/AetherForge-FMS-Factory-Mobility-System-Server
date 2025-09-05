-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username VARCHAR(100) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    role user_role NOT NULL DEFAULT 'operator',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create cameras table
CREATE TABLE cameras (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    device_id VARCHAR(100) NOT NULL UNIQUE,
    location VARCHAR(255) NOT NULL,
    stream_url TEXT NOT NULL,
    status camera_status NOT NULL DEFAULT 'offline',
    intrinsics JSONB,
    extrinsics JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create user roles enum
CREATE TYPE user_role AS ENUM ('admin', 'operator', 'viewer');

-- Create camera status enum
CREATE TYPE camera_status AS ENUM ('online', 'offline', 'calibrating', 'error');

-- Create camera calibrations history table
CREATE TABLE camera_calibrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    intrinsics JSONB NOT NULL,
    extrinsics JSONB NOT NULL,
    calibrated_by UUID NOT NULL REFERENCES users(id),
    calibrated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_cameras_status ON cameras(status);
CREATE INDEX idx_cameras_location ON cameras(location);
CREATE INDEX idx_calibrations_camera_id ON camera_calibrations(camera_id);
CREATE INDEX idx_calibrations_calibrated_at ON camera_calibrations(calibrated_at);



-- Create annotation status enum
CREATE TYPE annotation_status AS ENUM ('pending', 'completed', 'rejected');

-- Create annotations table
CREATE TABLE annotations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    image_path TEXT NOT NULL,
    camera_id UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    created_by UUID NOT NULL REFERENCES users(id),
    annotations JSONB NOT NULL,
    status annotation_status NOT NULL DEFAULT 'pending',
    reviewed BOOLEAN NOT NULL DEFAULT FALSE,
    reviewed_by UUID REFERENCES users(id),
    reviewed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create annotation tasks table
CREATE TABLE annotation_tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    image_path TEXT NOT NULL,
    camera_id UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_annotations_camera_id ON annotations(camera_id);
CREATE INDEX idx_annotations_status ON annotations(status);
CREATE INDEX idx_annotations_created_by ON annotations(created_by);
CREATE INDEX idx_annotation_tasks_camera_id ON annotation_tasks(camera_id);


-- Create model type enum
CREATE TYPE model_type AS ENUM ('object_detection', 'semantic_segmentation', 'instance_segmentation', 'classification');

-- Create model status enum
CREATE TYPE model_status AS ENUM ('draft', 'training', 'trained', 'validating', 'validated', 'deployed', 'archived');

-- Create models table
CREATE TABLE models (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    version VARCHAR(50) NOT NULL,
    model_path TEXT NOT NULL,
    model_type model_type NOT NULL,
    input_shape JSONB NOT NULL,
    output_shape JSONB NOT NULL,
    classes JSONB NOT NULL,
    performance_metrics JSONB,
    training_job_id UUID,
    status model_status NOT NULL DEFAULT 'draft',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create deployment status enum
CREATE TYPE deployment_status AS ENUM ('pending', 'deploying', 'active', 'failed', 'retiring', 'retired');

-- Create model deployments table
CREATE TABLE model_deployments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    deployed_to TEXT NOT NULL,
    status deployment_status NOT NULL DEFAULT 'pending',
    deployed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deployed_by UUID NOT NULL REFERENCES users(id)
);

-- Create indexes
CREATE INDEX idx_models_name ON models(name);
CREATE INDEX idx_models_status ON models(status);
CREATE INDEX idx_model_deployments_model_id ON model_deployments(model_id);
CREATE INDEX idx_model_deployments_status ON model_deployments(status);


-- Create training status enum
CREATE TYPE training_status AS ENUM ('pending', 'preparing', 'training', 'validating', 'completed', 'failed', 'cancelled');

-- Create training jobs table
CREATE TABLE training_jobs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    model_id UUID NOT NULL REFERENCES models(id) ON DELETE CASCADE,
    dataset_id UUID NOT NULL, -- Would reference a datasets table if we had one
    hyperparameters JSONB NOT NULL,
    status training_status NOT NULL DEFAULT 'pending',
    progress FLOAT NOT NULL DEFAULT 0,
    metrics JSONB,
    val_metrics JSONB,
    logs TEXT[] NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_training_jobs_model_id ON training_jobs(model_id);
CREATE INDEX idx_training_jobs_status ON training_jobs(status);
CREATE INDEX idx_training_jobs_created_by ON training_jobs(created_by);


-- Create system event type enum
CREATE TYPE system_event_type AS ENUM (
    'camera_offline',
    'camera_error',
    'inference_error',
    'training_error',
    'storage_low',
    'memory_high',
    'cpu_high',
    'service_down',
    'model_performance_degraded',
    'security_alert',
    'other'
);

-- Create event severity enum
CREATE TYPE event_severity AS ENUM ('critical', 'high', 'medium', 'low', 'info');

-- Create system events table
CREATE TABLE system_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type system_event_type NOT NULL,
    severity event_severity NOT NULL,
    message TEXT NOT NULL,
    details JSONB,
    source TEXT,
    acknowledged BOOLEAN NOT NULL DEFAULT FALSE,
    acknowledged_by UUID REFERENCES users(id),
    acknowledged_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_system_events_event_type ON system_events(event_type);
CREATE INDEX idx_system_events_severity ON system_events(severity);
CREATE INDEX idx_system_events_acknowledged ON system_events(acknowledged);
CREATE INDEX idx_system_events_created_at ON system_events(created_at);



-- Create camera health status enum
CREATE TYPE camera_health_status AS ENUM ('healthy', 'warning', 'critical', 'unknown');

-- Create calibration status enum
CREATE TYPE calibration_status AS ENUM ('not_calibrated', 'calibrating', 'calibrated', 'needs_recalibration', 'failed');

-- Create calibration pattern enum
CREATE TYPE calibration_pattern AS ENUM ('chessboard', 'circles', 'asymmetric_circles');

-- Add new columns to cameras table
ALTER TABLE cameras 
ADD COLUMN zone TEXT,
ADD COLUMN rtsp_url TEXT,
ADD COLUMN health_status camera_health_status NOT NULL DEFAULT 'unknown',
ADD COLUMN last_ping TIMESTAMPTZ,
ADD COLUMN fps FLOAT,
ADD COLUMN resolution_width INTEGER,
ADD COLUMN resolution_height INTEGER,
ADD COLUMN calibration_status calibration_status NOT NULL DEFAULT 'not_calibrated',
ADD COLUMN last_calibration TIMESTAMPTZ;

-- Create camera calibrations table
CREATE TABLE camera_calibrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    intrinsics JSONB NOT NULL,
    extrinsics JSONB NOT NULL,
    calibration_method TEXT NOT NULL,
    calibration_accuracy FLOAT NOT NULL,
    calibrated_by UUID NOT NULL REFERENCES users(id),
    calibrated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    calibration_images TEXT[] NOT NULL DEFAULT '{}'
);

-- Create camera health metrics table
CREATE TABLE camera_health_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    fps FLOAT NOT NULL,
    latency_ms FLOAT NOT NULL,
    packet_loss FLOAT NOT NULL,
    resolution_width INTEGER NOT NULL,
    resolution_height INTEGER NOT NULL,
    bitrate_kbps FLOAT NOT NULL,
    cpu_usage FLOAT NOT NULL,
    memory_usage FLOAT NOT NULL
);

-- Create camera status history table
CREATE TABLE camera_status_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    camera_id UUID NOT NULL REFERENCES cameras(id) ON DELETE CASCADE,
    status camera_status NOT NULL,
    health_status camera_health_status NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    message TEXT
);

-- Create camera zones table
CREATE TABLE camera_zones (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL UNIQUE,
    description TEXT,
    location TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_cameras_zone ON cameras(zone);
CREATE INDEX idx_cameras_health_status ON cameras(health_status);
CREATE INDEX idx_cameras_calibration_status ON cameras(calibration_status);
CREATE INDEX idx_camera_calibrations_camera_id ON camera_calibrations(camera_id);
CREATE INDEX idx_camera_health_metrics_camera_id ON camera_health_metrics(camera_id);
CREATE INDEX idx_camera_health_metrics_timestamp ON camera_health_metrics(timestamp);
CREATE INDEX idx_camera_status_history_camera_id ON camera_status_history(camera_id);
CREATE INDEX idx_camera_status_history_timestamp ON camera_status_history(timestamp);
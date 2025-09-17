"""
Member Events Handler - F-001 Welcome Message Feature
Handles GUILD_MEMBER_ADD events and sends welcome messages
"""
import asyncio
from typing import Dict, Any, Optional
from datetime import datetime

from ..services.config_service import ConfigService
from ..utils.logger import logger


class MemberEventHandler:
    """Handles Discord member events, specifically welcome messages for new members"""
    
    def __init__(self, config_service: ConfigService):
        """
        Initialize the member event handler
        
        Args:
            config_service: Service for retrieving guild configuration
        """
        self.config_service = config_service
        self._discord_client = None  # Will be injected by the event dispatcher
    
    def set_discord_client(self, client):
        """Set the Discord client for sending messages"""
        self._discord_client = client
    
    async def handle_guild_member_add(self, event_data: Dict[str, Any]) -> None:
        """
        Handle GUILD_MEMBER_ADD event by sending welcome message
        
        Args:
            event_data: Discord event data containing guild_id and user information
        """
        try:
            # Validate event data
            if not self._validate_event_data(event_data):
                logger.error(f"Invalid event data received: {event_data}")
                return
            
            guild_id = event_data['guild_id']
            user = event_data['user']
            
            # Check for duplicate events (idempotency)
            if self._is_duplicate_event(event_data):
                logger.info(f"Duplicate member join event for user {user['id']} in guild {guild_id}")
                return
            
            # Get welcome channel configuration
            welcome_channel_id = await self.config_service.get_welcome_channel(guild_id)
            
            if welcome_channel_id is None:
                logger.warning(f"No welcome channel configured for guild {guild_id}")
                return
            
            # Send welcome message
            await self._send_welcome_message(welcome_channel_id, user)
            
            logger.info(f"Welcome message sent for user {user['id']} in guild {guild_id}")
            
        except Exception as e:
            logger.error(f"Failed to process member join for guild {event_data.get('guild_id', 'unknown')}: {str(e)}")
    
    def _validate_event_data(self, event_data: Dict[str, Any]) -> bool:
        """
        Validate that event data contains required fields
        
        Args:
            event_data: Event data to validate
            
        Returns:
            True if valid, False otherwise
        """
        required_fields = ['guild_id', 'user']
        user_required_fields = ['id', 'username']
        
        # Check top-level fields
        for field in required_fields:
            if field not in event_data:
                return False
        
        # Check user fields
        user_data = event_data.get('user', {})
        for field in user_required_fields:
            if field not in user_data:
                return False
        
        return True
    
    def _is_duplicate_event(self, event_data: Dict[str, Any]) -> bool:
        """
        Check if this event has already been processed (idempotency check)
        
        Args:
            event_data: Event data to check
            
        Returns:
            True if duplicate, False otherwise
        """
        # TODO: Implement with dedup/idempotency component
        # For now, return False (no duplicate detection)
        return False
    
    async def _send_welcome_message(self, channel_id: str, user: Dict[str, str]) -> None:
        """
        Send welcome message to the specified channel
        
        Args:
            channel_id: Discord channel ID where to send the message
            user: User data containing id and username
        """
        try:
            channel = await self._get_channel(channel_id)
            if channel is None:
                logger.error(f"Welcome channel not found: {channel_id}")
                return
            
            # Construct welcome message with user mention
            user_id = user['id']
            username = user['username']
            message_content = f"歡迎 <@{user_id}> 加入伺服器！🎉 希望你在這裡玩得愉快！"
            
            # Send the message
            await channel.send(content=message_content)
            
            logger.info(f"Welcome message sent to channel {channel_id} for user {username} ({user_id})")
            
        except Exception as e:
            if "rate limit" in str(e).lower() or "429" in str(e):
                # Handle rate limiting
                retry_after = getattr(e, 'retry_after', 5.0)
                logger.warning(f"Rate limited, retrying after {retry_after} seconds")
                await asyncio.sleep(retry_after)
                # Could retry here, but for now just log
                return
            elif "permission" in str(e).lower() or "403" in str(e):
                logger.error(f"Permission denied to send message in channel {channel_id}")
                return
            else:
                logger.error(f"Failed to send welcome message: {str(e)}")
                raise
    
    async def _get_channel(self, channel_id: str):
        """
        Get Discord channel by ID
        
        Args:
            channel_id: Discord channel ID
            
        Returns:
            Discord channel object or None if not found
        """
        if self._discord_client is None:
            raise RuntimeError("Discord client not initialized")
        
        try:
            return await self._discord_client.fetch_channel(int(channel_id))
        except Exception as e:
            logger.error(f"Failed to fetch channel {channel_id}: {str(e)}")
            return None